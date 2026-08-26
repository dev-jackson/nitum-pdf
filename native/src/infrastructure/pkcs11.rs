use crate::domain::HardwareToken;
use anyhow::{Context, Result, bail};
use cryptoki::{
    context::{CInitializeArgs, CInitializeFlags, Pkcs11},
    mechanism::Mechanism,
    object::{Attribute, AttributeType, KeyType, ObjectClass, ObjectHandle},
    session::{Session, UserType},
    slot::Slot,
    types::AuthPin,
};
use std::sync::Mutex;
use underskrift::{
    CryptoSigner,
    crypto::algorithm::{DigestAlgorithm, SignatureAlgorithm},
    error::CryptoError,
};

pub struct Pkcs11Signer {
    session: Mutex<Session>,
    private_key: ObjectHandle,
    certificate: Vec<u8>,
    chain: Vec<Vec<u8>>,
    digest_algorithm: DigestAlgorithm,
    signature_algorithm: SignatureAlgorithm,
}

impl Pkcs11Signer {
    pub fn open(token: &HardwareToken, pin: &str) -> Result<Self> {
        let context =
            Pkcs11::new(&token.module_path).context("no se pudo cargar el módulo de la tarjeta")?;
        context.initialize(CInitializeArgs::new(CInitializeFlags::OS_LOCKING_OK))?;
        // Slot identifiers are not stable across PKCS#11 initializations or a
        // physical token reconnect. Resolve the selected token again by its
        // stable identity before falling back to the discovery-time slot id.
        let slot = context
            .get_slots_with_token()?
            .into_iter()
            .find(|slot| {
                context.get_token_info(*slot).is_ok_and(|info| {
                    info.serial_number().trim() == token.serial
                        && info.label().trim() == token.label
                })
            })
            .or_else(|| Slot::try_from(token.slot_id).ok())
            .context("el lector seleccionado ya no existe")?;
        let session = context.open_ro_session(slot)?;
        let pin = AuthPin::new(pin.to_owned().into());
        session
            .login(UserType::User, Some(&pin))
            .context("el PIN no es correcto o la tarjeta está bloqueada")?;

        let certificates = session.find_objects(&[Attribute::Class(ObjectClass::CERTIFICATE)])?;
        for certificate_handle in certificates {
            let attributes = session.get_attributes(
                certificate_handle,
                &[AttributeType::Id, AttributeType::Value],
            )?;
            let id = attributes.iter().find_map(|attribute| match attribute {
                Attribute::Id(value) => Some(value.clone()),
                _ => None,
            });
            let certificate = attributes.iter().find_map(|attribute| match attribute {
                Attribute::Value(value) => Some(value.clone()),
                _ => None,
            });
            let (Some(id), Some(certificate)) = (id, certificate) else {
                continue;
            };
            let keys = session.find_objects(&[
                Attribute::Class(ObjectClass::PRIVATE_KEY),
                Attribute::Id(id),
            ])?;
            if let Some(private_key) = keys.first() {
                let key_type = session
                    .get_attributes(*private_key, &[AttributeType::KeyType])?
                    .into_iter()
                    .find_map(|attribute| match attribute {
                        Attribute::KeyType(value) => Some(value),
                        _ => None,
                    });
                let (digest_algorithm, signature_algorithm) = match key_type {
                    Some(KeyType::RSA) => {
                        (DigestAlgorithm::Sha256, SignatureAlgorithm::RsaPkcs1v15)
                    }
                    Some(KeyType::EC) => {
                        let params = session
                            .get_attributes(*private_key, &[AttributeType::EcParams])?
                            .into_iter()
                            .find_map(|attribute| match attribute {
                                Attribute::EcParams(value) => Some(value),
                                _ => None,
                            });
                        match params.as_deref() {
                            Some([0x06, 0x08, 0x2a, 0x86, 0x48, 0xce, 0x3d, 0x03, 0x01, 0x07]) => {
                                (DigestAlgorithm::Sha256, SignatureAlgorithm::EcdsaP256)
                            }
                            Some([0x06, 0x05, 0x2b, 0x81, 0x04, 0x00, 0x22]) => {
                                (DigestAlgorithm::Sha384, SignatureAlgorithm::EcdsaP384)
                            }
                            _ => continue,
                        }
                    }
                    _ => continue,
                };
                return Ok(Self {
                    session: Mutex::new(session),
                    private_key: *private_key,
                    chain: vec![certificate.clone()],
                    certificate,
                    digest_algorithm,
                    signature_algorithm,
                });
            }
        }
        bail!(
            "La tarjeta no contiene un certificado RSA o ECDSA compatible con su clave privada asociada."
        )
    }
}

impl CryptoSigner for Pkcs11Signer {
    fn sign_hash(&self, hash: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let session = self
            .session
            .lock()
            .map_err(|_| CryptoError::SigningFailed("sesión PKCS#11 bloqueada".to_owned()))?;
        match self.signature_algorithm {
            SignatureAlgorithm::RsaPkcs1v15 => {
                const SHA256_PREFIX: &[u8] = &[
                    0x30, 0x31, 0x30, 0x0d, 0x06, 0x09, 0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04,
                    0x02, 0x01, 0x05, 0x00, 0x04, 0x20,
                ];
                const SHA384_PREFIX: &[u8] = &[
                    0x30, 0x41, 0x30, 0x0d, 0x06, 0x09, 0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04,
                    0x02, 0x02, 0x05, 0x00, 0x04, 0x30,
                ];
                let mut digest_info = match self.digest_algorithm {
                    DigestAlgorithm::Sha256 => SHA256_PREFIX.to_vec(),
                    DigestAlgorithm::Sha384 => SHA384_PREFIX.to_vec(),
                    _ => {
                        return Err(CryptoError::UnsupportedKeyType(
                            "hash RSA no compatible".to_owned(),
                        ));
                    }
                };
                digest_info.extend_from_slice(hash);
                session
                    .sign(&Mechanism::RsaPkcs, self.private_key, &digest_info)
                    .map_err(|error| CryptoError::SigningFailed(error.to_string()))
            }
            SignatureAlgorithm::EcdsaP256 | SignatureAlgorithm::EcdsaP384 => {
                let raw = session
                    .sign(&Mechanism::Ecdsa, self.private_key, hash)
                    .map_err(|error| CryptoError::SigningFailed(error.to_string()))?;
                ecdsa_raw_to_der(&raw)
            }
            _ => Err(CryptoError::UnsupportedKeyType(
                "algoritmo PKCS#11 no compatible".to_owned(),
            )),
        }
    }

    fn certificate_der(&self) -> &[u8] {
        &self.certificate
    }

    fn certificate_chain_der(&self) -> Vec<&[u8]> {
        self.chain.iter().map(Vec::as_slice).collect()
    }

    fn digest_algorithm(&self) -> DigestAlgorithm {
        self.digest_algorithm
    }

    fn signature_algorithm(&self) -> SignatureAlgorithm {
        self.signature_algorithm
    }
}

fn ecdsa_raw_to_der(raw: &[u8]) -> Result<Vec<u8>, CryptoError> {
    if raw.is_empty() || !raw.len().is_multiple_of(2) {
        return Err(CryptoError::SigningFailed(
            "firma ECDSA inválida del token".to_owned(),
        ));
    }
    fn integer(value: &[u8]) -> Vec<u8> {
        let first = value
            .iter()
            .position(|byte| *byte != 0)
            .unwrap_or(value.len() - 1);
        let value = &value[first..];
        let needs_zero = value[0] & 0x80 != 0;
        let mut result = vec![0x02, (value.len() + usize::from(needs_zero)) as u8];
        if needs_zero {
            result.push(0);
        }
        result.extend_from_slice(value);
        result
    }
    let (r, s) = raw.split_at(raw.len() / 2);
    let r = integer(r);
    let s = integer(s);
    let mut result = vec![0x30, (r.len() + s.len()) as u8];
    result.extend_from_slice(&r);
    result.extend_from_slice(&s);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_fixed_width_ecdsa_to_canonical_der() {
        let mut raw = vec![0_u8; 64];
        raw[31] = 1;
        raw[32] = 0x80;
        let der = ecdsa_raw_to_der(&raw).unwrap();
        assert_eq!(&der[..5], &[0x30, 0x26, 0x02, 0x01, 0x01]);
        assert_eq!(der[5..8], [0x02, 0x21, 0x00]);
    }
}
