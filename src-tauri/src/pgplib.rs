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
use crate::password_cache;

#[tauri::command]
pub fn generate_keypair(
    app_handle: tauri::AppHandle,
    user_id: &str,
    passphrase: &str,
) -> Result<(), LibError> {
    println!("Generating keypair for user: {}", user_id);
    // println!("Passphrase: {}", passphrase); // REMOVE sensitive println

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
        .primary_user_id(user_id.to_string())
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
pub fn encrypt_message(
    app_handle: tauri::AppHandle,
    plain_text: &str,
    public_key: Option<String>
) -> Result<String, LibError> {
    let public_key = if let Some(pubkey_str) = public_key {
        SignedPublicKey::from_string(&pubkey_str)?.0
    } else {
        let app_data_path = app_handle
            .path()
            .app_data_dir()
            .unwrap();
        let public_key_path = app_data_path.join("public_key.asc");
        let public_key_str = std::fs::read_to_string(public_key_path)?;
        SignedPublicKey::from_string(&public_key_str)?.0
    };

    let message = Message::new_literal_bytes("", plain_text.as_bytes());
    let compressed_msg = message.compress(CompressionAlgorithm::ZLIB).unwrap();
    let encrypted_msg = compressed_msg
        .encrypt_to_keys_seipdv1(
            thread_rng(),
            pgp::crypto::sym::SymmetricKeyAlgorithm::AES256,
            &[&public_key],
        )
        .unwrap();

    println!("✓ Encryption successful");
    let encrypted_message = encrypted_msg.to_armored_string(None.into()).unwrap();
    Ok(encrypted_message)
}

// #[tauri::command]
// pub fn is_cache_valid() -> bool {
//     password_cache::get().is_some()
// }

#[tauri::command]
pub fn decrypt_message(
    app_handle: tauri::AppHandle,
    encrypted_text: &str,
    passphrase: Option<String>
) -> Result<String, LibError> {
    // If passphrase is provided, cache it; otherwise, use cached password
    let passphrase = match passphrase {
        Some(ref p) => {
            password_cache::set(p.clone());
            p.clone()
        },
        None => password_cache::get().ok_or(LibError::Custom("Master password required".to_string()))?,
    };

    let app_data_path = app_handle
        .path()
        .app_data_dir()
        .unwrap();

    println!("Come to decrypt....");
    // println!("Encrypted Text: {}", encrypted_text); // REMOVE sensitive println

    let private_key_path = app_data_path.join("private_key.asc");
    println!("Private Key path (before creation): {}", private_key_path.display());
    // println!("Private Key: {}", private_key_str); // REMOVE sensitive println
    let private_key_str = std::fs::read_to_string(private_key_path)?;
    let private_key = SignedSecretKey::from_string(&private_key_str)?.0;
    println!("Private Key loaded successfully.");

    // Gracefully handle error when parsing armored message
    let message_result = Message::from_armor_single(encrypted_text.as_bytes());
    let message = match message_result {
        Ok((msg, _)) => msg,
        Err(e) => {
            println!("Error parsing armored message: {:?}", e);
            return Err(LibError::Custom(format!("Failed to parse encrypted message: {:?}", e)));
        }
    };

    println!("Encrypted text: {}", encrypted_text);
    println!("Decrypting message with passphrase: [REDACTED]");
    // println!("Private Key: {}", private_key_str); // REMOVE sensitive println

    // Decrypt the message with the private key
    let decrypted_message: (Message, Vec<pgp::types::KeyId>) = match message.decrypt(|| passphrase.to_string(), &[&private_key]) {
        Ok(dm) => dm,
        Err(e) => {
            println!("Error decrypting message: {:?}", e);
            return Err(LibError::Custom(format!("Failed to decrypt message: {:?}", e)));
        }
    };

    println!("Message successfully decrypted.");

    let decompressed = match decrypted_message.0.decompress() {
        Ok(d) => d,
        Err(e) => {
            println!("Error decompressing message: {:?}", e);
            return Err(LibError::Custom(format!("Failed to decompress message: {:?}", e)));
        }
    };

    let content = match decompressed.get_content() {
        Ok(Some(c)) => c,
        Ok(None) => {
            println!("No content found in decrypted message.");
            return Err(LibError::Custom("No content found in decrypted message".to_string()));
        }
        Err(e) => {
            println!("Error getting content: {:?}", e);
            return Err(LibError::Custom(format!("Failed to get content: {:?}", e)));
        }
    };

    Ok(String::from_utf8(content).unwrap_or_else(|e| {
        println!("Error converting decrypted content to UTF-8: {:?}", e);
        String::from("[Invalid UTF-8]")
    }))
}

#[tauri::command]
pub fn get_email_ids_from_public_key(pubkey: String) -> Result<String, String> {
    let public_key = SignedPublicKey::from_string(&pubkey).unwrap().0;
    let user_ids: Vec<String> = public_key
        .details
        .users
        .iter()
        .map(|uid| uid.id.clone().id().to_string())
        .collect();
    let found_user_id = user_ids[0].clone();
    Ok(found_user_id)
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
        let decrypted_message: (Message, Vec<pgp::types::KeyId>) = message
            .decrypt(|| passphrase.to_string(), &[&private_key])
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

    #[test]
    fn test_read_email_from_public_key() {
        let temp_dir = TempDir::new().unwrap();
        let app_data_path = temp_dir.path().join("app_data");
        println!("Test App Data Path: {:?}", app_data_path);
        fs::create_dir_all(&app_data_path).unwrap();
        
        let user_id = "test@example.com";
        let passphrase = "test_passphrase";
     
        // Create test keys
        create_test_keys(&app_data_path, user_id, passphrase);
        // Read the public key from the specified file
        let public_key_path = app_data_path.join("public_key.asc");
        let public_key_str = std::fs::read_to_string(public_key_path).unwrap();
        let public_key = SignedPublicKey::from_string(&public_key_str).unwrap().0;
        let user_ids: Vec<String> = public_key
            .details
            .users
            .iter()
            .map(|uid| uid.id.clone().id().to_string())
            .collect();

        let found_user_id = user_ids[0].clone();
        println!("User ID found in public key: {}", found_user_id);   

        assert!(user_ids.contains(&user_id.to_string()));
        println!("✓ Read email from public key test passed!");
    }
}

