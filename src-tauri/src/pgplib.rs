use pgp::composed::{KeyType, SecretKeyParamsBuilder};
use pgp::types::{CompressionAlgorithm, SecretKeyTrait};
use pgp::{Deserializable, Message, SignedPublicKey, SignedSecretKey};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use rand::thread_rng;

use crate::libutils::LibError;

#[tauri::command]
pub fn generate_keypair(user_id: &str, passphrase: &str, path: &Path) -> Result<(), LibError> {
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

    let signed_public_key = public_key.sign(thread_rng(), &signed_secret_key, || passphrase.to_string()).unwrap();

    // Save the secret key to a file
    let secret_key_path = path.join("private_key.asc");
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
pub fn encrypt_message(public_key_path: &Path, plain_text: &str) -> Result<String, LibError> {
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
            .encrypt_to_keys_seipdv1(thread_rng(), pgp::crypto::sym::SymmetricKeyAlgorithm::AES256, &[&public_key])
            .unwrap();
        
    let encrypted_message = encrypted_msg.to_armored_string(None.into()).unwrap();

    Ok(encrypted_message)
}

#[tauri::command]
pub fn decrypt_message(private_key_path: &Path, encrypted_text: &str) -> Result<String, LibError> {
    // Read the private key from the specified file
    let private_key_str = std::fs::read_to_string(private_key_path)?;
    let private_key = SignedSecretKey::from_string(&private_key_str)?.0;

    // Decrypt the message with the private key
    let message = Message::from_armor_single(encrypted_text.as_bytes()).unwrap().0;

    let passphrase = "alphagamma";
    
    let decrypted_message: (Message, Vec<pgp::types::KeyId>) = message
        .decrypt(|| passphrase.to_string(), &[&private_key])
        .unwrap();

    let decrypted_message = decrypted_message.0.decompress().unwrap();
    let decrypted_message = decrypted_message.get_content().unwrap().unwrap();

    Ok(String::from_utf8(decrypted_message).unwrap())
}