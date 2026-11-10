//! Ed25519 signatures used by QQ Bot webhooks and interactions.

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};

use crate::error::{Error, Result};

/// Header containing the Ed25519 signature.
pub const SIGNATURE_HEADER: &str = "X-Signature-Ed25519";
/// Header containing the signature timestamp.
pub const TIMESTAMP_HEADER: &str = "X-Signature-Timestamp";

/// Signs `timestamp || body` with a key derived from `secret`.
///
/// # Errors
///
/// Returns an error when the secret is empty.
pub fn sign(secret: &str, timestamp: &str, body: &[u8]) -> Result<String> {
    if timestamp.is_empty() {
        return Err(Error::Signature("timestamp is empty".to_owned()));
    }
    let key = signing_key(secret)?;
    let mut message = Vec::with_capacity(timestamp.len() + body.len());
    message.extend_from_slice(timestamp.as_bytes());
    message.extend_from_slice(body);
    Ok(hex::encode(key.sign(&message).to_bytes()))
}

/// Verifies a hex-encoded signature over `timestamp || body`.
///
/// # Errors
///
/// Returns an error when the secret, signature, or timestamp is malformed, or
/// when the signature is invalid.
pub fn verify(secret: &str, signature: &str, timestamp: &str, body: &[u8]) -> Result<()> {
    if signature.is_empty() {
        return Err(Error::Signature("signature is empty".to_owned()));
    }
    if timestamp.is_empty() {
        return Err(Error::Signature("timestamp is empty".to_owned()));
    }

    let signature_bytes = hex::decode(signature)
        .map_err(|error| Error::Signature(format!("signature is not valid hex: {error}")))?;
    let signature = Signature::from_slice(&signature_bytes)
        .map_err(|error| Error::Signature(format!("signature is not valid: {error}")))?;

    let key = verifying_key(secret)?;
    let mut message = Vec::with_capacity(timestamp.len() + body.len());
    message.extend_from_slice(timestamp.as_bytes());
    message.extend_from_slice(body);

    key.verify(&message, &signature)
        .map_err(|_| Error::Signature("ed25519 verification failed".to_owned()))
}

/// Returns `true` when the signature is valid.
///
/// This helper is convenient in HTTP handlers where an invalid signature is
/// expected to produce a `401` rather than an error value.
#[must_use]
pub fn is_valid(secret: &str, signature: &str, timestamp: &str, body: &[u8]) -> bool {
    verify(secret, signature, timestamp, body).is_ok()
}

fn signing_key(secret: &str) -> Result<SigningKey> {
    let seed = seed_from_secret(secret)?;
    Ok(SigningKey::from_bytes(&seed))
}

fn verifying_key(secret: &str) -> Result<VerifyingKey> {
    Ok(signing_key(secret)?.verifying_key())
}

fn seed_from_secret(secret: &str) -> Result<[u8; ed25519_dalek::SECRET_KEY_LENGTH]> {
    if secret.is_empty() {
        return Err(Error::Signature("secret is empty".to_owned()));
    }

    let mut seed = secret.to_owned();
    while seed.len() < ed25519_dalek::SECRET_KEY_LENGTH {
        seed.push_str(&seed.clone());
    }
    let bytes = seed.as_bytes();
    let mut output = [0_u8; ed25519_dalek::SECRET_KEY_LENGTH];
    output.copy_from_slice(&bytes[..ed25519_dalek::SECRET_KEY_LENGTH]);
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &str = "123456abcdef";
    const TIMESTAMP: &str = "1728981195";
    const BODY: &str = r#"{"id":"ROBOT1.0_veoihSEXDc8Q.g-6eLpNIa11bH8MisOjn-m-LKxCPntMk6exUXgcWCGpVO7L2QKTNZzjZzFFDSbiOFcqAPWyVA!!","content":"哦一下","timestamp":"2024-10-15T16:33:15+08:00","author":{"id":"675860273","user_openid":"675860273"}}"#;
    const SIGNATURE: &str = "e949b5b94ef4103df903fb031d1d16e358db3db83e79e117edd404c8508be3ce8a76d7bad1bed353194c126a1a5915b4ad8b5288c1191cc53a12acffccd82004";

    #[test]
    fn verifies_the_go_sdk_test_vector() {
        verify(SECRET, SIGNATURE, TIMESTAMP, BODY.as_bytes()).unwrap();
    }

    #[test]
    fn rejects_a_modified_body() {
        assert!(verify(SECRET, SIGNATURE, TIMESTAMP, b"modified").is_err());
    }

    #[test]
    fn signing_round_trips() {
        let signature = sign(SECRET, TIMESTAMP, BODY.as_bytes()).unwrap();
        assert_eq!(signature, SIGNATURE);
    }
}
