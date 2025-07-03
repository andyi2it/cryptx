//Check a key file exists in tauri's app directoty

import { exists, BaseDirectory } from '@tauri-apps/plugin-fs';
import { invoke } from '@tauri-apps/api/core';

export async function checkKeyFileExists() {
  const fileName = `private_key.asc`;
  var fileExists = await exists(fileName, {baseDir: BaseDirectory.AppData});
  console.log('Checking if key file exists at', fileExists);
  return fileExists;

}

export async function init(email: string, password: string) {
    const keyFileExists = await checkKeyFileExists();
    if (!keyFileExists) {
        console.error('Key file does not exist');
        try {
            await invoke('generate_keypair', { "userId": email, "passphrase": password });
            console.log('Key pair generated');
        } catch (error) {
            console.error('Failed to generate key pair', error);
            return;
        }
    } else {
        console.log('Key file exists');
    }
}

