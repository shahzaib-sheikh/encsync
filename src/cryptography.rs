use anyhow::anyhow;
use chacha20poly1305::{
    aead::{stream, Aead, NewAead},
    XChaCha20Poly1305,
};
use rand::{rngs::OsRng, RngCore};
use std::{
    fs::{self, File},
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
    let mut dist_file = File::create(dist_file_path)?;

    loop {
        let read_count = source_file.read(&mut buffer)?;

        if read_count == BUFFER_LEN {
            let ciphertext = stream_encryptor
                .encrypt_next(buffer.as_slice())
                .map_err(|err| anyhow!("Encrypting large file: {}", err))?;
            dist_file.write(&ciphertext)?;
        } else {
            let ciphertext = stream_encryptor
                .encrypt_last(&buffer[..read_count])
                .map_err(|err| anyhow!("Encrypting large file: {}", err))?;
            dist_file.write(&ciphertext)?;
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
        let mut file = File::create(temp_file.path()).unwrap();
        file.write_all(plaintext.as_bytes()).unwrap();

        // Encrypt the file
        let key = "my secret key";
        let nonce: [u8; 19] = [0; 19];
        encrypt_file(temp_file.path(), key, &nonce).unwrap();

        // Read the encrypted file and verify its contents
        let mut encrypted_file = File::open(temp_file.path()).unwrap();
        let mut encrypted_contents = String::new();
        encrypted_file
            .read_to_string(&mut encrypted_contents)
            .unwrap();
        assert_ne!(encrypted_contents, plaintext);

        // Decrypt the file and verify its contents
        let mut decrypted_contents = String::new();
        decrypt_file(temp_file.path(), key)
            .unwrap()
            .read_to_string(&mut decrypted_contents)
            .unwrap();
        assert_eq!(decrypted_contents, plaintext);
    }

    #[test]
    fn test_decrypt_file() {
        // Create a temporary file to decrypt
        let plaintext = "Hello, world!";
        let temp_file = NamedTempFile::new().unwrap();
        let mut file = File::create(temp_file.path()).unwrap();
        file.write_all(plaintext.as_bytes()).unwrap();

        // Encrypt the file
        let key = "my secret key";
        encrypt_file(temp_file.path(), key).unwrap();

        // Decrypt the file and verify its contents
        let mut decrypted_contents = String::new();
        decrypt_file(temp_file.path(), key)
            .unwrap()
            .read_to_string(&mut decrypted_contents)
            .unwrap();
        assert_eq!(decrypted_contents, plaintext);
    }

    #[test]
    fn test_encrypt_file_with_invalid_key() {
        // Create a temporary file to encrypt
        let plaintext = "Hello, world!";
        let temp_file = NamedTempFile::new().unwrap();
        let mut file = File::create(temp_file.path()).unwrap();
        file.write_all(plaintext.as_bytes()).unwrap();

        // Encrypt the file with an invalid key
        let key = "";
        let result = encrypt_file(temp_file.path(), key);

        // Verify that the encryption failed
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_file_with_invalid_key() {
        // Create a temporary file to decrypt
        let plaintext = "Hello, world!";
        let temp_file = NamedTempFile::new().unwrap();
        let mut file = File::create(temp_file.path()).unwrap();
        file.write_all(plaintext.as_bytes()).unwrap();

        // Encrypt the file
        let key = "my secret key";
        encrypt_file(temp_file.path(), key).unwrap();

        // Decrypt the file with an invalid key
        let key = "";
        let result = decrypt_file(temp_file.path(), key);

        // Verify that the decryption failed
        assert!(result.is_err());
    }
}
