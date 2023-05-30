use anyhow::anyhow;
use chacha20poly1305::{aead::stream, KeyInit, XChaCha20Poly1305};
use rand::{rngs::OsRng, RngCore};
use std::{
    fs::File,
    io::{Read, Write},
};

// source: https://github.com/skerkour/kerkour.com/blob/8a0f6782e151d86dbcafc95050fcc2654d8cf4d3/blog/2021/rust_file_encryption/src/main.rs#LL111C1-L145C2
fn encrypt_file(
    source_file_path: &str,
    dist_file_path: &str,
    key: &[u8; 32],
    nonce: &[u8; 19],
) -> Result<(), anyhow::Error> {
    let aead = XChaCha20Poly1305::new(key.as_ref().into());
    let mut stream_encryptor = stream::EncryptorBE32::from_aead(aead, nonce.as_ref().into());

    const BUFFER_LEN: usize = 500;
    let mut buffer = [0u8; BUFFER_LEN];

    let mut source_file = File::open(source_file_path)?;
    let mut dest_file = File::create(dist_file_path)?;

    loop {
        let read_count = source_file.read(&mut buffer)?;

        if read_count == BUFFER_LEN {
            let cipher_text = stream_encryptor
                .encrypt_next(&buffer[..read_count])
                .map_err(|err| anyhow!("Encrypting large file: {}", err))?;

            dest_file.write(&cipher_text)?;
        } else {
            let cipher_text = stream_encryptor
                .encrypt_last(&buffer[..read_count])
                .map_err(|err| anyhow!("Encrypting large file: {}", err))?;
            dest_file.write(&cipher_text)?;
            break;
        }
    }

    Ok(())
}

// source: https://github.com/skerkour/kerkour.com/blob/8a0f6782e151d86dbcafc95050fcc2654d8cf4d3/blog/2021/rust_file_encryption/src/main.rs#LL147C1-L182C2
fn decrypt_file(
    encrypted_file_path: &str,
    dist: &str,
    key: &[u8; 32],
    nonce: &[u8; 19],
) -> Result<(), anyhow::Error> {
    let aead = XChaCha20Poly1305::new(key.as_ref().into());
    let mut stream_decryptor = stream::DecryptorBE32::from_aead(aead, nonce.as_ref().into());

    const BUFFER_LEN: usize = 500 + 16;
    let mut buffer = [0u8; BUFFER_LEN];

    let mut encrypted_file = File::open(encrypted_file_path)?;
    let mut dist_file = File::create(dist)?;

    loop {
        let read_count = encrypted_file.read(&mut buffer)?;

        if read_count == BUFFER_LEN {
            let plaintext = stream_decryptor
                .decrypt_next(buffer.as_slice())
                .map_err(|err| anyhow!("Decrypting large file: {}", err))?;
            dist_file.write(&plaintext)?;
        } else if read_count == 0 {
            break;
        } else {
            let plaintext = stream_decryptor
                .decrypt_last(&buffer[..read_count])
                .map_err(|err| anyhow!("Decrypting large file: {}", err))?;
            dist_file.write(&plaintext)?;
            break;
        }
    }

    Ok(())
}

pub fn generate_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}

pub fn generate_nonce() -> [u8; 19] {
    let mut nonce = [0u8; 19];
    OsRng.fill_bytes(&mut nonce);
    nonce
}

// ==================== TESTS ==================== //
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::{Read, Write};
    use tempfile::NamedTempFile;

    #[test]
    fn test_encrypt_file() {
        // Create a temporary file to encrypt
        let plaintext = "Hello, world!";
        let temp_file = NamedTempFile::new().unwrap();
        let temp_file2 = NamedTempFile::new().unwrap();
        let mut file = File::create(temp_file.path()).unwrap();

        file.write_all(plaintext.as_bytes()).unwrap();

        let mut plain_text_file = File::open(temp_file.path()).unwrap();

        let mut plain_text_contents = String::new();
        plain_text_file
            .read_to_string(&mut plain_text_contents)
            .unwrap();

        assert_eq!(plain_text_contents, plaintext);

        // Encrypt the file
        let key = generate_key();
        let nonce = generate_nonce();

        encrypt_file(
            temp_file.path().to_str().unwrap(),
            temp_file2.path().to_str().unwrap(),
            &key,
            &nonce,
        )
        .unwrap();

        // Decrypt the file and verify its contents
        let temp_file3 = NamedTempFile::new().unwrap();

        decrypt_file(
            temp_file2.path().to_str().unwrap(),
            temp_file3.path().to_str().unwrap(),
            &key,
            &nonce,
        )
        .unwrap();

        let mut decrypted_file = File::open(temp_file3.path()).unwrap();
        let mut decrypted_contents = String::new();
        decrypted_file
            .read_to_string(&mut decrypted_contents)
            .unwrap();

        assert_eq!(decrypted_contents, plaintext);
    }

    #[test]
    fn test_decrypt_file_with_invalid_key() {
        // Create a temporary file to encrypt
        let plaintext = "Hello, world!";
        let temp_file = NamedTempFile::new().unwrap();
        let temp_file2 = NamedTempFile::new().unwrap();
        let mut file = File::create(temp_file.path()).unwrap();

        file.write_all(plaintext.as_bytes()).unwrap();

        // Encrypt the file
        let key = generate_key();
        let nonce = generate_nonce();
        encrypt_file(
            temp_file.path().to_str().unwrap(),
            temp_file2.path().to_str().unwrap(),
            &key,
            &nonce,
        )
        .unwrap();

        // Decrypt the file and verify its contents
        let temp_file3 = NamedTempFile::new().unwrap();

        let key2 = generate_key();
        let result = decrypt_file(
            temp_file2.path().to_str().unwrap(),
            temp_file3.path().to_str().unwrap(),
            &key2,
            &nonce,
        );

        // Verify that the decryption failed
        assert!(result.is_err());
    }
}
