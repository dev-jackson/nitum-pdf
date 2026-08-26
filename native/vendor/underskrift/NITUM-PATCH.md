# Nitum compatibility patch

This source is pinned from `underskrift` commit
`3a953ef41043c41c653b7194a155f4c226c4f370`.

Nitum additionally verifies CMS signatures that use the generic
`rsaEncryption` signature OID by selecting SHA-256, SHA-384, or SHA-512 from
the SignerInfo digest algorithm. RFC 3161 responses from DigiCert use this
standards-compatible encoding.
