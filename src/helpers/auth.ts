import { invoke } from '@tauri-apps/api/core';

interface LoginCredentials {
  email: string;
  passwordHash: string;
  createdAt: string;
}

// Simple hash function for demo purposes (in production, use proper hashing)
async function hashPassword(password: string): Promise<string> {
  const encoder = new TextEncoder();
  const data = encoder.encode(password);
  const hashBuffer = await crypto.subtle.digest('SHA-256', data);
  const hashArray = Array.from(new Uint8Array(hashBuffer));
  return hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
}

async function getCredentialsPath(): Promise<string> {
  try {
    const appDataDir = await invoke('get_app_data_dir') as string;
    return await invoke('join_path', { base: appDataDir, segment: 'login_credentials.json' }) as string;
  } catch (error) {
    console.error('Failed to get app data directory:', error);
    throw error;
  }
}

export async function saveLoginCredentials(email: string, password: string): Promise<void> {
  try {
    const credentialsPath = await getCredentialsPath();
    const passwordHash = await hashPassword(password);
    
    const credentials: LoginCredentials = {
      email,
      passwordHash,
      createdAt: new Date().toISOString()
    };
    
    await invoke('write_text_file', { 
      path: credentialsPath, 
      content: JSON.stringify(credentials, null, 2) 
    });
    console.log('Login credentials saved successfully');
  } catch (error) {
    console.error('Failed to save login credentials:', error);
    throw error;
  }
}

export async function validateLoginCredentials(email: string, password: string): Promise<boolean> {
  try {
    const credentialsPath = await getCredentialsPath();
    
    const exists = await invoke('file_exists', { path: credentialsPath }) as boolean;
    if (!exists) {
      console.log('No stored credentials found');
      return false;
    }
    
    const credentialsContent = await invoke('read_text_file', { path: credentialsPath }) as string;
    const storedCredentials: LoginCredentials = JSON.parse(credentialsContent);
    
    const passwordHash = await hashPassword(password);
    
    return storedCredentials.email === email && storedCredentials.passwordHash === passwordHash;
  } catch (error) {
    console.error('Failed to validate login credentials:', error);
    return false;
  }
}

export async function hasStoredCredentials(): Promise<boolean> {
  try {
    const credentialsPath = await getCredentialsPath();
    return await invoke('file_exists', { path: credentialsPath }) as boolean;
  } catch (error) {
    console.error('Failed to check for stored credentials:', error);
    return false;
  }
}

export async function getStoredEmail(): Promise<string | null> {
  try {
    const credentialsPath = await getCredentialsPath();
    
    const exists = await invoke('file_exists', { path: credentialsPath }) as boolean;
    if (!exists) {
      return null;
    }
    
    const credentialsContent = await invoke('read_text_file', { path: credentialsPath }) as string;
    const storedCredentials: LoginCredentials = JSON.parse(credentialsContent);
    
    return storedCredentials.email;
  } catch (error) {
    console.error('Failed to get stored email:', error);
    return null;
  }
}
