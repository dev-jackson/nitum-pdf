//! Signature and DocTimeStamp dictionary creation.
//!
//! Creates `/Type /Sig` and `/Type /DocTimeStamp` dictionaries with proper
//! `/Filter`, `/SubFilter`, `/ByteRange`, and `/Contents` entries.

use lopdf::{Dictionary, Object};

/// SubFilter values we support.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SigSubFilter {
    /// PAdES: `/ETSI.CAdES.detached`
    EtsiCadesDetached,
    /// Traditional: `/adbe.pkcs7.detached`
    AdbePkcs7Detached,
}

impl SigSubFilter {
    /// Returns the PDF name string for this sub-filter.
    pub fn as_pdf_name(&self) -> &'static str {
        match self {
            SigSubFilter::EtsiCadesDetached => "ETSI.CAdES.detached",
            SigSubFilter::AdbePkcs7Detached => "adbe.pkcs7.detached",
        }
    }
}

/// Build a signature dictionary with placeholder ByteRange and Contents.
///
/// `contents_size` is the number of bytes to reserve for the hex-encoded
/// signature in `/Contents`. This must be large enough to hold the final
/// CMS signature.
pub fn build_sig_dict(sub_filter: SigSubFilter, contents_size: usize) -> Dictionary {
    let mut dict = Dictionary::new();
    dict.set("Type", Object::Name(b"Sig".to_vec()));
    dict.set("Filter", Object::Name(b"Adobe.PPKLite".to_vec()));
    dict.set(
        "SubFilter",
        Object::Name(sub_filter.as_pdf_name().as_bytes().to_vec()),
    );

    // ETSI EN 319 142 requires the PDF signature dictionary's /M entry for a
    // PAdES Baseline-B signature. It is only a claimed signing time (the
    // cryptographic proof of time comes from RFC 3161), but omitting it makes
    // interoperable validators classify an otherwise valid signature as the
    // legacy PAdES-BES profile.
    let signing_time = chrono::Utc::now()
        .format("D:%Y%m%d%H%M%S+00'00'")
        .to_string();
    dict.set(
        "M",
        Object::String(signing_time.into_bytes(), lopdf::StringFormat::Literal),
    );

    // ByteRange placeholder — will be backpatched after serialization
    // Using [0 0 0 0] as placeholder; real values computed during incremental write
    dict.set(
        "ByteRange",
        Object::Array(vec![
            Object::Integer(0),
            Object::Integer(0),
            Object::Integer(0),
            Object::Integer(0),
        ]),
    );

    // Contents placeholder — hex-encoded zeroes, sized to `contents_size`
    // The actual signature bytes will replace these zeroes after signing
    let placeholder = vec![0u8; contents_size];
    dict.set(
        "Contents",
        Object::String(placeholder, lopdf::StringFormat::Hexadecimal),
    );

    dict
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baseline_dictionary_contains_a_pdf_signing_time() {
        let dict = build_sig_dict(SigSubFilter::EtsiCadesDetached, 256);
        let signing_time = dict.get(b"M").unwrap().as_str().unwrap();

        assert!(signing_time.starts_with(b"D:"));
        assert!(signing_time.ends_with(b"+00'00'"));
    }
}
