use age::secrecy::ExposeSecret;
use age::secrecy::SecretBox;
use age::secrecy::SecretString;
use age::x25519;
use age::Encryptor;
use std::io::Write;
use std::fs::File;
use std::path::Path;
use std::str::FromStr;

// ...existing code...

pub fn create_key_pair(password: &str, private_key_path: &Path, public_key_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Generate a new key pair
    let key_pair = x25519::Identity::generate();

    let private_key_str = key_pair.to_string();
    println!("Private key: {}", private_key_str.expose_secret());

    // Create secretbox from password
    let passphrase: SecretBox<str> = SecretString::from(password.to_owned());

    // Encrypt the private key with the provided password
    let encryptor: Encryptor = Encryptor::with_user_passphrase(passphrase);
    let mut encrypted_private_key = vec![];
    encryptor.wrap_output(&mut encrypted_private_key)?.write_all(key_pair.to_string().expose_secret().as_bytes())?;

    // Write the encrypted private key to the specified file
    let mut private_key_file = File::create(private_key_path)?;
    private_key_file.write_all(&encrypted_private_key)?;

    // Write the public key to the specified file
    let mut public_key_file = File::create(public_key_path)?;
    public_key_file.write_all(key_pair.to_public().to_string().as_bytes())?;

    Ok(())
}


pub fn encrypt_message(public_key_path: &Path, message: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Read the public key from the specified file
    let public_key_str = std::fs::read_to_string(public_key_path)?;
    let public_key = x25519::Recipient::from_str(&public_key_str)?;

    // Encrypt the message with the public key
    let encrypted_message = age::encrypt(&public_key, message.as_bytes())?;

    Ok(encrypted_message)
}

pub fn decrypt_message(private_key_path: &Path, encrypted_text: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Read the private key from the specified file
    let private_key_str = std::fs::read_to_string(private_key_path)?;
    let private_key = x25519::Identity::from_str(&private_key_str)?;

    // Decrypt the message with the private key
    let decrypted_message = age::decrypt(&private_key, encrypted_text.as_bytes())?;
    Ok(String::from_utf8(decrypted_message)?)
}
