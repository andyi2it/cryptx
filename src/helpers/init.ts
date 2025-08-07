//Check a key file exists in tauri's app directoty

import { exists, BaseDirectory } from '@tauri-apps/plugin-fs';
import { invoke } from '@tauri-apps/api/core';

export async function checkKeyFileExists() {
  const fileName = `private_key.asc`;
  var fileExists = await exists(fileName, {baseDir: BaseDirectory.AppData});
  console.log('Checking if key file exists at', fileExists);
  return fileExists;

}

// window.initPGP = init;  // Attach it to the window

// declare global {
//   interface Window {
//     initPGP: (email: string, password: string) => Promise<void>;
//   }
// }

export async function init(email: string, password: string) {
    try {
        console.log('Starting init process for:', email);
        const keyFileExists = await checkKeyFileExists();
        if (!keyFileExists) {
            console.log('Key file does not exist, generating new keypair...');
            try {
                await invoke('generate_keypair', { "userId": email, "passphrase": password });
                console.log('Key pair generated successfully');
                
                // Verify the files were created
                const keyFileExistsAfter = await checkKeyFileExists();
                console.log('Key file exists after generation:', keyFileExistsAfter);
            } catch (error) {
                console.error('Failed to generate key pair:', error);
                throw error;
            }
        } else {
            console.log('Key file already exists');
        }
    } catch (error) {
        console.error('Init process failed:', error);
        throw error;
    }
}

