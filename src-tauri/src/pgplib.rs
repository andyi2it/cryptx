use pgp::composed::{KeyType, SecretKeyParamsBuilder};
use pgp::types::{CompressionAlgorithm, SecretKeyTrait};
use pgp::{Deserializable, Message, SignedPublicKey, SignedSecretKey};
use rand::thread_rng;
use tauri::Manager;
//use tauri::path::app_data_dir;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::libutils::LibError;

#[tauri::command]
pub fn generate_keypair(
    app_handle: tauri::AppHandle,
    userId: &str,
    passphrase: &str,
) -> Result<(), LibError> {
    println!("Generating keypair for user: {}", userId);
    println!("Passphrase: {}", passphrase);

    let app_data_path = app_handle
        .path()
        .app_data_dir()
        .unwrap()
        .display()
        .to_string();
    println!("App Data Path: {}", app_data_path);
    let path = Path::new(&app_data_path);
    
    // Generate secret key parameters
    let secret_key_params = SecretKeyParamsBuilder::default()
        .key_type(KeyType::Rsa(2048))
        .primary_user_id(userId.to_string())
        .passphrase(Some(passphrase.to_string()))
        .build()?;

    // Generate the secret key
    let secret_key = secret_key_params.generate(thread_rng())?;

    let signed_secret_key = secret_key.sign(thread_rng(), || passphrase.to_string())?;

    // Extract the public key from the secret key
    let public_key = signed_secret_key.public_key();

    let signed_public_key = public_key
        .sign(thread_rng(), &signed_secret_key, || passphrase.to_string())
        .unwrap();

    // Save the secret key to a file
    let secret_key_path = path.join("private_key.asc");
    println!("Secret Key path (before creation): {}", secret_key_path.display());

    let mut secret_key_file = File::create(&secret_key_path)?;
    let bytes = signed_secret_key.to_armored_bytes(None.into())?;
    secret_key_file.write_all(&bytes)?;
    
    println!("Secret Key created successfully at: {}", secret_key_path.display());

    // Save the public key to a file
    let public_key_path = path.join("public_key.asc");
    println!("Public Key path (before creation): {}", public_key_path.display());
    
    let mut public_key_file = File::create(&public_key_path)?;
    let pub_bytes = signed_public_key.to_armored_bytes(None.into())?;
    public_key_file.write_all(&pub_bytes)?;
    
    println!("Public Key created successfully at: {}", public_key_path.display());
    println!("Keypair generation completed successfully!");

    Ok(())
}

#[tauri::command]
pub fn encrypt_message(app_handle: tauri::AppHandle, plain_text: &str) -> Result<String, LibError> {
    let app_data_path = app_handle
        .path()
        .app_data_dir()
        .unwrap();
    let public_key_path = app_data_path.join("public_key.asc");
    
    // Read the public key from the specified file
    let public_key_str = std::fs::read_to_string(public_key_path)?;
    let public_key = SignedPublicKey::from_string(&public_key_str)?.0;

    //Convert PkesBytes to Vec<u8>
    //let message = Message::from_bytes(plain_text.as_bytes()).unwrap();
    //let data = message.compress(CompressionAlgorithm::ZLIB).unwrap();
    //let enc_data = data.encrypt_to_keys_seipdv1(thread_rng(), pgp::crypto::sym::SymmetricKeyAlgorithm::AES256, &[&public_key]).unwrap();

    let message = Message::new_literal_bytes("", plain_text.as_bytes());
    let compressed_msg = message.compress(CompressionAlgorithm::ZLIB).unwrap();
    let encrypted_msg = compressed_msg
        .encrypt_to_keys_seipdv1(
            thread_rng(),
            pgp::crypto::sym::SymmetricKeyAlgorithm::AES256,
            &[&public_key],
        )
        .unwrap();

    let encrypted_message = encrypted_msg.to_armored_string(None.into()).unwrap();

    Ok(encrypted_message)
}

#[tauri::command]
pub fn decrypt_message(app_handle: tauri::AppHandle, encrypted_text: &str, passphrase: &str) -> Result<String, LibError> {
    let app_data_path = app_handle
        .path()
        .app_data_dir()
        .unwrap();
    // let private_key_path = app_data_path.join("private_key.asc");
    // // add print statement for passphrase
    // println!("Decrypting message with passphrase: {}", passphrase);
    // // Read the private key from the specified file
    // let private_key_str = std::fs::read_to_string(private_key_path)?;
    // println!("Private key string: {}", private_key_str);
    // let private_key = SignedSecretKey::from_string(&private_key_str)?.0;
    // println!("Private key successfully loaded.");

    // // Decrypt the message with the private key
    // let message = Message::from_armor_single(encrypted_text.as_bytes())
    //     .map_err(|e| LibError::PgpError(pgp::errors::Error::Message(format!("Failed to parse encrypted message: {}", e))))?
    //     .0;

    // println!("Message successfully parsed.");

    // let decrypted_message: (Message, Vec<pgp::types::KeyId>) = message
    //     .decrypt(|| passphrase.to_string(), &[&private_key])
    //     .map_err(|e| LibError::PgpError(pgp::errors::Error::Message(format!("Failed to decrypt message (check passphrase and key): {}", e))))?;

    // println!("Message successfully decrypted.");
    // let decrypted_message = decrypted_message.0.decompress()
    //     .map_err(|e| LibError::PgpError(pgp::errors::Error::Message(format!("Failed to decompress message: {}", e))))?;
    
    // println!("Message successfully decompressed.");
    // let decrypted_content = decrypted_message.get_content()
    //     .map_err(|e| LibError::PgpError(pgp::errors::Error::Message(format!("Failed to get message content: {}", e))))?
    //     .ok_or_else(|| LibError::PgpError(pgp::errors::Error::Message("Message content is empty".to_string())))?;

    // println!("Message content successfully retrieved.");
    // String::from_utf8(decrypted_content)
    //     .map_err(|e| LibError::PgpError(pgp::errors::Error::Message(format!("Failed to convert message to UTF-8: {}", e))))
    let private_key_path = app_data_path.join("private_key.asc");
    let private_key_str = std::fs::read_to_string(private_key_path)?;
    let private_key = SignedSecretKey::from_string(&private_key_str)?.0;

    // Decrypt the message with the private key
    let message = Message::from_armor_single(encrypted_text.as_bytes()).unwrap().0;
    println!("Encrypted text: {}", encrypted_text);
    // let passphrase = "alphagamma";
    println!("Decrypting message with passphrase: {}", passphrase);
    let decrypted_message: (Message, Vec<pgp::types::KeyId>) = message
        .decrypt(|| passphrase.to_string(), &[&private_key])
        .unwrap();
    
    println!("Message successfully decrypted.");

    let decrypted_message = decrypted_message.0.decompress().unwrap();
    let decrypted_message = decrypted_message.get_content().unwrap().unwrap();

    Ok(String::from_utf8(decrypted_message).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_keys(app_data_path: &std::path::Path, user_id: &str, passphrase: &str) {
        let user_id = user_id.to_string();
        let passphrase = passphrase.to_string();
        
        // Generate secret key parameters
        let secret_key_params = SecretKeyParamsBuilder::default()
            .key_type(KeyType::Rsa(2048))
            .primary_user_id(user_id.clone())
            .passphrase(Some(passphrase.clone()))
            .build().unwrap();

        // Generate the secret key
        let secret_key = secret_key_params.generate(thread_rng()).unwrap();
        let signed_secret_key = secret_key.sign(thread_rng(), || passphrase.clone()).unwrap();

        // Extract the public key from the secret key
        let public_key = signed_secret_key.public_key();
        let signed_public_key = public_key
            .sign(thread_rng(), &signed_secret_key, || passphrase.clone())
            .unwrap();

        // Save the secret key to a file
        let secret_key_path = app_data_path.join("private_key.asc");
        let mut secret_key_file = File::create(&secret_key_path).unwrap();
        let bytes = signed_secret_key.to_armored_bytes(None.into()).unwrap();
        secret_key_file.write_all(&bytes).unwrap();

        // Save the public key to a file
        let public_key_path = app_data_path.join("public_key.asc");
        let mut public_key_file = File::create(&public_key_path).unwrap();
        let pub_bytes = signed_public_key.to_armored_bytes(None.into()).unwrap();
        public_key_file.write_all(&pub_bytes).unwrap();
    }

    #[test]
    fn test_encrypt_message() {
        let temp_dir = TempDir::new().unwrap();
        let app_data_path = temp_dir.path().join("app_data");
        println!("Test App Data Path: {:?}", app_data_path);
        fs::create_dir_all(&app_data_path).unwrap();
        
        let user_id = "test@example.com";
        let passphrase = "test_passphrase";
        let plain_text = "Hello, this is a test message for encryption!";
        
        // Create test keys
        create_test_keys(&app_data_path, user_id, passphrase);
        
        // Test encrypt_message logic
        let public_key_path = app_data_path.join("public_key.asc");
        let public_key_str = std::fs::read_to_string(public_key_path).unwrap();
        let public_key = SignedPublicKey::from_string(&public_key_str).unwrap().0;

        let message = Message::new_literal_bytes("", plain_text.as_bytes());
        let compressed_msg = message.compress(CompressionAlgorithm::ZLIB).unwrap();
        let encrypted_msg = compressed_msg
            .encrypt_to_keys_seipdv1(
                thread_rng(),
                pgp::crypto::sym::SymmetricKeyAlgorithm::AES256,
                &[&public_key],
            )
            .unwrap();
        let encrypted_message = encrypted_msg.to_armored_string(None.into()).unwrap();
        
        println!("✓ Encryption successful");
        println!("Original message: {}", plain_text);
        println!("Encrypted message length: {}", encrypted_message.len());
        
        // Verify encryption output format
        assert!(encrypted_message.contains("-----BEGIN PGP MESSAGE-----"));
        assert!(encrypted_message.contains("-----END PGP MESSAGE-----"));
        assert!(encrypted_message.len() > 100);
        assert_ne!(encrypted_message, plain_text);
        
        println!("✓ Encrypt message test passed!");
    }

    #[test]
    fn test_decrypt_message() {
        let temp_dir = TempDir::new().unwrap();
        let app_data_path = temp_dir.path().join("app_data");
        println!("Test App Data Path: {:?}", app_data_path);
        fs::create_dir_all(&app_data_path).unwrap();
        
        let user_id = "test@example.com";
        let passphrase = "test_pasphrase";
        let plain_text = "Hello, this is a test message for decryption!";
        
        // Create test keys
        create_test_keys(&app_data_path, user_id, passphrase);
        
        // First encrypt a message (setup for decrypt test)
        let public_key_path = app_data_path.join("public_key.asc");
        let public_key_str = std::fs::read_to_string(&public_key_path).unwrap();
        let public_key = SignedPublicKey::from_string(&public_key_str).unwrap().0;

        let message = Message::new_literal_bytes("", plain_text.as_bytes());
        let compressed_msg = message.compress(CompressionAlgorithm::ZLIB).unwrap();
        let encrypted_msg = compressed_msg
            .encrypt_to_keys_seipdv1(
                thread_rng(),
                pgp::crypto::sym::SymmetricKeyAlgorithm::AES256,
                &[&public_key],
            )
            .unwrap();
        let encrypted_message = encrypted_msg.to_armored_string(None.into()).unwrap();
        
        // Now test decrypt_message logic
        let private_key_path = app_data_path.join("private_key.asc");
        let private_key_str = std::fs::read_to_string(private_key_path).unwrap();
        let private_key = SignedSecretKey::from_string(&private_key_str).unwrap().0;

        let message = Message::from_armor_single(encrypted_message.as_bytes()).unwrap().0;
        println!("Encrypted text: {}", encrypted_message);
        println!("Decrypting message with passphrase: {}", passphrase);
        let passphrase1 = 's';
        let decrypted_message: (Message, Vec<pgp::types::KeyId>) = message
            .decrypt(|| passphrase1.to_string(), &[&private_key])
            .unwrap();

        let decrypted_message = decrypted_message.0.decompress().unwrap();
        let decrypted_content = decrypted_message.get_content().unwrap().unwrap();
        let decrypted_text = String::from_utf8(decrypted_content).unwrap();
        
        println!("✓ Decryption successful");
        println!("Original message: {}", plain_text);
        println!("Decrypted message: {}", decrypted_text);
        
        // Verify decryption result
        assert_eq!(decrypted_text, plain_text);
        assert_ne!(decrypted_text, encrypted_message);
        
        println!("✓ Decrypt message test passed!");
    }
}

