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
    println!("Secret Key path is: {:?}", secret_key_path.canonicalize());

    let mut secret_key_file = File::create(&secret_key_path)?;
    let bytes = signed_secret_key.to_armored_bytes(None.into())?;
    secret_key_file.write_all(&bytes)?;

    // Save the public key to a file
    let public_key_path = path.join("public_key.asc");
    let mut public_key_file = File::create(&public_key_path)?;

    let pub_bytes = signed_public_key.to_armored_bytes(None.into())?;
    public_key_file.write_all(&pub_bytes)?;

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
    let private_key_path = app_data_path.join("private_key.asc");
    // add print statement for passphrase
    println!("Decrypting message with passphrase: {}", passphrase);
    // Read the private key from the specified file
    let private_key_str = std::fs::read_to_string(private_key_path)?;
    println!("Private key string: {}", private_key_str);
    let private_key = SignedSecretKey::from_string(&private_key_str)?.0;
    println!("Private key successfully loaded.");

    // Decrypt the message with the private key
    let message = Message::from_armor_single(encrypted_text.as_bytes())
        .map_err(|e| LibError::PgpError(pgp::errors::Error::Message(format!("Failed to parse encrypted message: {}", e))))?
        .0;

    println!("Message successfully parsed.");

    let decrypted_message: (Message, Vec<pgp::types::KeyId>) = message
        .decrypt(|| passphrase.to_string(), &[&private_key])
        .map_err(|e| LibError::PgpError(pgp::errors::Error::Message(format!("Failed to decrypt message (check passphrase and key): {}", e))))?;

    println!("Message successfully decrypted.");
    let decrypted_message = decrypted_message.0.decompress()
        .map_err(|e| LibError::PgpError(pgp::errors::Error::Message(format!("Failed to decompress message: {}", e))))?;
    
    println!("Message successfully decompressed.");
    let decrypted_content = decrypted_message.get_content()
        .map_err(|e| LibError::PgpError(pgp::errors::Error::Message(format!("Failed to get message content: {}", e))))?
        .ok_or_else(|| LibError::PgpError(pgp::errors::Error::Message("Message content is empty".to_string())))?;

    println!("Message content successfully retrieved.");
    String::from_utf8(decrypted_content)
        .map_err(|e| LibError::PgpError(pgp::errors::Error::Message(format!("Failed to convert message to UTF-8: {}", e))))
}
