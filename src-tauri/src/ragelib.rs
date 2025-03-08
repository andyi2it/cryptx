// Create a library to generate a private key and public key pair and save it to a file
use rage::cli_common::generate::generate_keypair;
use std::fs;
use std::io;

const IDENTITY_FILE_PATH: &str = "my_identity.txt"; // Path to the identity file

// Function to check if the identity file exists
pub fn ensure_identity_file_exists(file_path: &str) -> io::Result<()> {
    if !std::path::Path::new(file_path).exists() {
        println!("Identity file not found. Creating one...");
        
        // Generate an age key pair
        let keypair = generate_keypair();
        
        // Serialize the keypair to an identity file format
        let identity_content = keypair.to_string();
        
        // Save the identity to the specified file path
        fs::write(file_path, identity_content)?;
        println!("Identity file created at: {}", file_path);
    } else {
        println!("Identity file already exists at: {}", file_path);
    }
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .setup(|_| {
            // Ensure identity file exists at app startup
            if let Err(e) = ensure_identity_file_exists(IDENTITY_FILE_PATH) {
                eprintln!("Failed to ensure identity file: {}", e);
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}