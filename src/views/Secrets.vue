<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { addSecret, getSecrets, getUsers, initDatabase, updateSecret, deleteSecret as dbDeleteSecret } from '../lib/database';
import { invoke } from '@tauri-apps/api/core';

const search = ref('');
const secrets = ref<Array<{ 
  id: number; 
  name: string; 
  key: string; 
  created_at: string;
  updated_at: string;
  shared_with?: number[];
  is_shared?: boolean;
}>>([]);

const filteredSecrets = computed(() => {
  return secrets.value.filter(secret => 
    secret.name.toLowerCase().includes(search.value.toLowerCase())
  );
});

// Import/Create dialog state
const importDialog = ref(false);
const valid = ref(false);
const form = ref();
const newSecret = ref({
  name: '',
  key: ''
});
const isCreateMode = ref(false);

// Share dialog state
const shareDialog = ref(false);
const selectedSecretForShare = ref<any>(null);
const userSearch = ref('');
const selectedUserForShare = ref<number | null>(null);
const users = ref<Array<{ id: number; name: string; email: string; public_key: string }>>([]);

const filteredUsersForShare = computed(() => {
  return users.value.filter(user => 
    user.name.toLowerCase().includes(userSearch.value.toLowerCase())
  );
});

// Secret details dialog state
const secretDetailsDialog = ref(false);
const selectedSecretDetails = ref<{ id: number; name: string; key: string; created_at: string; updated_at: string } | null>(null);
const showSecretKey = ref(false);
const isEditingSecret = ref(false);
const editableSecret = ref({ name: '', key: '' });

// Confirmation dialog state
const confirmDeleteDialog = ref(false);
const secretToDelete = ref<{ id: number; name: string; key: string } | null>(null);

// Password prompt dialog state
const passwordPromptDialog = ref(false);
const masterPassword = ref('');
const passwordPromptValid = ref(false);
const passwordForm = ref();
const isValidatingPassword = ref(false);
const passwordErrorMessage = ref('');

// Encrypted message dialog state
const encryptedMessageDialog = ref(false);
const encryptedMessage = ref('');
const isEncrypting = ref(false);
const encryptError = ref('');
const copySuccess = ref(false);

// Removed getCachedMasterPassword and cacheMasterPassword helpers

const isValidPGPMessage = (value: string) => {
  const trimmed = value.trim();
  return (
    trimmed.startsWith('-----BEGIN PGP MESSAGE-----') &&
    trimmed.endsWith('-----END PGP MESSAGE-----')
  ) || 'Invalid PGP message format';
};

const saveSecret = async () => {
  console.log("Save secret called, form valid:", valid.value);
  console.log("New secret data:", newSecret.value);
  
  if (!valid.value) {
    console.error("Form validation failed");
    alert("Please fix the form errors before saving");
    return;
  }
  
  if (!newSecret.value.name || !newSecret.value.key) {
    console.error("Missing required fields");
    alert("Please fill in all required fields");
    return;
  }
  
  try {
    console.log("Attempting to save secret:", newSecret.value.name);

    let encryptedKey: string;
    if (isCreateMode.value) {
      // Encrypt the secret value before storing in database (create mode)
      console.log("Encrypting secret...");
      console.log("key:", newSecret.value.key);
      encryptedKey = await invoke('encrypt_message', {
        plainText: newSecret.value.key
      }) as string;
      console.log("Secret encrypted successfully");
    } else {
      // Import mode: secret is already encrypted
      encryptedKey = newSecret.value.key;
      console.log("Imported secret, skipping encryption");
    }

    console.log("Encrypted key:", encryptedKey);
    // Save the secret with encrypted value
    await addSecret(newSecret.value.name, encryptedKey);
    console.log("Secret saved successfully");
    await loadSecrets();
    closeImportDialog();
  } catch (error) {
    console.error('Detailed save error:', error);
    const errorMessage = error instanceof Error ? error.message : String(error);
    alert(`Failed to save secret: ${errorMessage}`);
  }
};

const loadSecrets = async () => {
  try {
    console.log("Loading secrets from database...");
    const loadedSecrets = await getSecrets();
    console.log("Loaded secrets:", loadedSecrets);
    // loop loaded secrets and print key
    loadedSecrets.forEach(secret => {
      console.log("Secret name:", secret.name);
      console.log("Secret key:", secret.key);
    });

    // Add mock data for shared status
    secrets.value = loadedSecrets.map(secret => ({
      ...secret,
      shared_with: Math.random() > 0.7 ? [1, 2] : [],
      is_shared: Math.random() > 0.7
    }));
    
    console.log("Secrets processed and set to reactive state");
  } catch (error) {
    console.error('Failed to load secrets:', error);
    // Don't show alert on initial load, just log the error
    console.warn("Continuing with empty secrets list");
    secrets.value = [];
  }
};

const loadUsers = async () => {
  try {
    users.value = await getUsers();
  } catch (error) {
    console.error('Failed to load users:', error);
    // Don't show alert, just log and continue
    users.value = [];
  }
};

const openImportDialog = () => {
  isCreateMode.value = false;
  importDialog.value = true;
};

const openCreateDialog = () => {
  isCreateMode.value = true;
  importDialog.value = true;
};

const closeImportDialog = () => {
  importDialog.value = false;
  newSecret.value = { name: '', key: '' };
  isCreateMode.value = false;
  if (form.value) {
    form.value.reset();
  }
};

const openShareDialog = (secret: any) => {
  selectedSecretForShare.value = secret;
  selectedUserForShare.value = secret.shared_with?.[0] || null;
  shareDialog.value = true;
  userSearch.value = '';
};

const closeShareDialog = () => {
  shareDialog.value = false;
  selectedSecretForShare.value = null;
  userSearch.value = '';
  selectedUserForShare.value = null;
};

const shareSecret = async () => {
  console.log("Share secret called");
  if (selectedSecretForShare.value && selectedUserForShare.value) {
    console.log("Selected secret:", selectedSecretForShare.value);
    console.log("Selected user:", selectedUserForShare.value);

    // Check if master password is cached in backend
    const cacheValid = await invoke<boolean>('is_cache_valid');
    if (cacheValid) {
      try {
        // Use cached password by passing null
        const decryptedSecret = await invoke('decrypt_message', {
          encryptedText: selectedSecretForShare.value.key.trim(),
          passphrase: null
        }) as string;

        // Find the selected user's public key
        const user = users.value.find(u => u.id === selectedUserForShare.value);
        if (!user) {
          encryptError.value = 'User not found';
          return;
        }

        // Encrypt with user's public key
        console.log("Encrypting secret for user:", user.name);
        console.log("Decrypted secret:", decryptedSecret);
        console.log("User's public key:", user.public_key);

        const encrypted = await invoke('encrypt_message', {
          plainText: decryptedSecret,
          publicKey: user.public_key
        }) as string;

        encryptedMessage.value = encrypted;
        shareDialog.value = false;
        encryptedMessageDialog.value = true;
        passwordPromptDialog.value = false;
        masterPassword.value = '';
        passwordErrorMessage.value = '';
        return;
      } catch (error) {
        // If cached password fails, fallback to prompt
        console.log("Cached password failed for sharing, fallback to prompt.");
      }
    }

    // Show password prompt if no cached password or decryption failed
    passwordPromptDialog.value = true;
    encryptError.value = '';
    masterPassword.value = '';
    return;
  }
};

const proceedShareAfterUnlock = async () => {
  if (
    selectedSecretForShare.value &&
    selectedUserForShare.value &&
    masterPassword.value &&
    passwordPromptValid.value
  ) {
    isEncrypting.value = true;
    encryptError.value = '';
    encryptedMessage.value = '';
    try {
      // Decrypt the secret using the master password
      const decryptedSecret = await invoke('decrypt_message', {
        encryptedText: selectedSecretForShare.value.key.trim(),
        passphrase: masterPassword.value
      }) as string;

      // Find the selected user's public key
      const user = users.value.find(u => u.id === selectedUserForShare.value);
      if (!user) {
        encryptError.value = 'User not found';
        isEncrypting.value = false;
        // Only close password dialog if it was open
        passwordPromptDialog.value = false;
        masterPassword.value = '';
        passwordErrorMessage.value = '';
        return;
      }

      // Encrypt with user's public key
      const encrypted = await invoke('encrypt_message', {
        plainText: decryptedSecret,
        publicKey: user.public_key
      }) as string;

      encryptedMessage.value = encrypted;
      shareDialog.value = false;
      encryptedMessageDialog.value = true;
      passwordPromptDialog.value = false;
      masterPassword.value = '';
      passwordErrorMessage.value = '';
    } catch (error) {
      // Show error in password dialog and keep it open
      passwordErrorMessage.value = 'Invalid master password. Please try again.';
      // Do NOT close passwordPromptDialog here
      masterPassword.value = '';
      // Optionally, clear encryptError so it doesn't show in the share dialog
      encryptError.value = '';
      return;
    } finally {
      isEncrypting.value = false;
    }
  }
};

const closeEncryptedMessageDialog = () => {
  encryptedMessageDialog.value = false;
  encryptedMessage.value = '';
  encryptError.value = '';
};

const toggleSecretVisibility = async () => {
  if (!showSecretKey.value) {
    // Check if master password is cached in backend
    const cacheValid = await invoke<boolean>('is_cache_valid');
    if (cacheValid) {
      if (selectedSecretDetails.value) {
        try {
          // Use cached password by passing null
          const decryptedSecret = await invoke('decrypt_message', {
            encryptedText: selectedSecretDetails.value.key.trim(),
            passphrase: null
          }) as string;
          editableSecret.value.key = decryptedSecret;
          showSecretKey.value = true;
          return;
        } catch (error) {
          // If cached password fails, fallback to prompt
        }
      }
    }
    // Show password prompt if no cached password or decryption failed
    passwordPromptDialog.value = true;
    masterPassword.value = '';
  } else {
    // Hide the secret
    showSecretKey.value = false;
  }
};

const validateMasterPassword = async () => {
  if (!passwordPromptValid.value || !masterPassword.value) {
    return;
  }

  try {
    isValidatingPassword.value = true;
    passwordErrorMessage.value = '';

    if (selectedSecretDetails.value) {
      try {
        const decryptedSecret = await invoke('decrypt_message', {
          encryptedText: selectedSecretDetails.value.key.trim(),
          passphrase: masterPassword.value
        }) as string;

        editableSecret.value.key = decryptedSecret;
        showSecretKey.value = true;
        passwordPromptDialog.value = false;
        passwordErrorMessage.value = '';
        // No need to cache the password here, Rust will handle it
        masterPassword.value = '';
      } catch (decryptError) {
        passwordErrorMessage.value = 'Invalid master password. Please try again.';
      }
    }
  } catch (error) {
    passwordErrorMessage.value = 'Invalid master password or corrupted secret. Please try again.';
  } finally {
    isValidatingPassword.value = false;
  }
};

// const validatePasswordHash = async (password: string, storedHash: string): Promise<boolean> => {
//   try {
//     // Use proper hash function from auth helper
//     const encoder = new TextEncoder();
//     const data = encoder.encode(password);
//     const hashBuffer = await crypto.subtle.digest('SHA-256', data);
//     const hashArray = Array.from(new Uint8Array(hashBuffer));
//     const passwordHash = hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
    
//     return passwordHash === storedHash;
//   } catch (error) {
//     console.error('Password validation error:', error);
//     return false;
//   }
// };

const cancelPasswordPrompt = () => {
  passwordPromptDialog.value = false;
  masterPassword.value = '';
  passwordErrorMessage.value = '';
  if (passwordForm.value) {
    passwordForm.value.reset();
  }
};

const openSecretDetails = (secret: { id: number; name: string; key: string; created_at: string; updated_at: string }) => {
  selectedSecretDetails.value = secret;
  // Always reset editableSecret to encrypted value when opening details
  editableSecret.value = { name: secret.name, key: secret.key };
  isEditingSecret.value = false;
  showSecretKey.value = false;
  secretDetailsDialog.value = true;
};

const closeSecretDetails = () => {
  secretDetailsDialog.value = false;
  selectedSecretDetails.value = null;
  showSecretKey.value = false;
  isEditingSecret.value = false;
};

const startEditingSecret = () => {
  isEditingSecret.value = true;
};

const cancelEditingSecret = () => {
  if (selectedSecretDetails.value) {
    editableSecret.value = { 
      name: selectedSecretDetails.value.name, 
      key: selectedSecretDetails.value.key 
    };
  }
  isEditingSecret.value = false;
};

const saveSecretChanges = async () => {
  if (selectedSecretDetails.value && editableSecret.value.name && editableSecret.value.key) {
    try {
      console.log("Attempting to encrypt and update secret:", selectedSecretDetails.value.id);
      
      // Encrypt the secret value before updating in database
      const encryptedKey = await invoke('encrypt_message', {
        plainText: editableSecret.value.key
      });
      
      console.log("Secret encrypted successfully for update");
      
      await updateSecret(selectedSecretDetails.value.id, editableSecret.value.name, encryptedKey as string);
      console.log("Secret updated successfully");
      await loadSecrets();
      isEditingSecret.value = false;
      closeSecretDetails();
    } catch (error) {
      console.error('Failed to update secret:', error);
      const errorMessage = error instanceof Error ? error.message : 'Unknown error occurred';
      alert(`Failed to update secret: ${errorMessage}`);
    }
  } else {
    alert("Please fill in all required fields");
  }
};

const formatDate = (dateString: string | undefined) => {
  if (!dateString) return 'Unknown';
  
  // Parse the UTC date string and convert to local time
  const utcDate = new Date(dateString + (dateString.includes('Z') ? '' : 'Z'));
  
  return utcDate.toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
    // timeZoneName: 'short'
  });
};

const secretNameRules = [
  (v: string) => !!v || 'Secret name is required',
  (v: string) => {
    const existingSecret = secrets.value.find(secret => 
      secret.name.toLowerCase() === v.toLowerCase()
    );
    return !existingSecret || 'Secret name already exists';
  }
];

const encryptedSecretRules = [
  (v: string) => !!v || 'Encrypted secret value is required',
  isValidPGPMessage
];

const masterPasswordRules = [
  (v: string) => !!v || 'Master password is required'
];

onMounted(async () => {
  try {
    console.log("Component mounted, initializing...");
    await initDatabase();
    console.log("Database initialized successfully");
    await loadSecrets();
    await loadUsers();
    console.log("Initial data loading completed");
  } catch (error) {
    console.error("Failed to initialize app:", error);
    // Show error but don't block the app
    console.warn("App initialization had issues, but continuing...");
  }
});

const confirmDeleteSecret = (secret: { id: number; name: string; key: string }) => {
  secretToDelete.value = secret;
  confirmDeleteDialog.value = true;
};

const deleteSecret = async () => {
  if (secretToDelete.value) {
    try {
      await dbDeleteSecret(secretToDelete.value.id);
      await loadSecrets();
    } catch (error) {
      console.error('Failed to delete secret:', error);
      alert('Failed to delete secret: ' + error);
    } finally {
      confirmDeleteDialog.value = false;
      secretToDelete.value = null;
    }
  }
};

const cancelDelete = () => {
  confirmDeleteDialog.value = false;
  secretToDelete.value = null;
};

// Password dialog "Verify" button logic
// Only call proceedShareAfterUnlock if sharing, otherwise call validateMasterPassword for reveal
const handlePasswordDialogVerify = async () => {
  if (shareDialog.value) {
    await proceedShareAfterUnlock();
  } else {
    await validateMasterPassword();
  }
};

const copyEncryptedMessage = async () => {
  try {
    // Use the raw encryptedMessage value, which includes newlines
    await navigator.clipboard.writeText(encryptedMessage.value);
    copySuccess.value = true;
    setTimeout(() => {
      copySuccess.value = false;
    }, 1500);
  } catch (err) {
    copySuccess.value = false;
    alert('Failed to copy message');
  }
};
</script>

<template>
  <v-container fluid class="pa-0">
    <!-- Sticky Search Bar -->
    <v-card class="sticky-search ma-4 mb-2" elevation="2">
      <v-card-text class="pb-2">
        <v-text-field
          v-model="search"
          placeholder="Search secrets..."
          prepend-inner-icon="mdi-magnify"
          variant="outlined"
          hide-details
          clearable
          density="comfortable"
        />
      </v-card-text>
    </v-card>

    <!-- Add Secret Buttons -->
    <div class="px-4 pb-2 d-flex gap-2">
      <v-btn
        color="primary"
        prepend-icon="mdi-plus"
        @click="openCreateDialog"
        variant="elevated"
      >
        Create Secret
      </v-btn>
      <v-btn
        color="secondary"
        prepend-icon="mdi-import"
        @click="openImportDialog"
        variant="outlined"
      >
        Import Secret
      </v-btn>
    </div>

    <!-- Secrets List -->
    <div class="secrets-list px-4">
      <div v-if="filteredSecrets.length === 0" class="text-center py-8">
        <v-icon size="64" color="grey-lighten-1">mdi-shield-key-outline</v-icon>
        <h3 class="mt-4 text-grey">No secrets found</h3>
        <p class="text-grey">Create your first secret to get started</p>
      </div>

      <v-card
        v-for="secret in filteredSecrets"
        :key="secret.id"
        class="mb-4 secret-card"
        elevation="2"
        hover
      >
        <v-card-text class="pa-4">
          <div class="d-flex justify-space-between align-start">
            <div class="flex-grow-1" @click="openSecretDetails(secret)" style="cursor: pointer;">
              <!-- Secret name with privacy indicator -->
              <div class="d-flex align-center mb-2">
                <v-icon
                  :icon="secret.is_shared ? 'mdi-account-group' : 'mdi-lock'"
                  :color="secret.is_shared ? 'primary' : 'grey'"
                  class="mr-2"
                  size="20"
                />
                <h4 class="text-h6">{{ secret.name }}</h4>
                <v-chip
                  v-if="secret.is_shared && secret.shared_with?.length"
                  size="small"
                  color="primary"
                  variant="outlined"
                  class="ml-2"
                >
                  {{ secret.shared_with.length }} user{{ secret.shared_with.length > 1 ? 's' : '' }}
                </v-chip>
              </div>

              <!-- Timestamps -->
              <div class="d-flex flex-column text-caption text-grey mb-2">
                <div class="d-flex align-center">
                  <v-icon size="14" class="mr-1">mdi-calendar-plus</v-icon>
                  Created: {{ formatDate(secret.created_at) }}
                </div>
                <div class="d-flex align-center mt-1">
                  <v-icon size="14" class="mr-1">mdi-calendar-edit</v-icon>
                  Updated: {{ formatDate(secret.updated_at) }}
                </div>
              </div>
            </div>

            <!-- Action buttons - always visible with proper styling -->
            <div class="action-buttons d-flex align-center ml-4">
              <v-btn
                size="small"
                variant="elevated"
                color="primary"
                @click.stop="openShareDialog(secret)"
                class="action-btn mr-2"
              >
                SHARE
              </v-btn>
              <v-btn
                size="small"
                variant="elevated"
                color="error"
                @click.stop="confirmDeleteSecret(secret)"
                class="action-btn"
              >
                DELETE
              </v-btn>
            </div>
          </div>
        </v-card-text>
      </v-card>
    </div>

    <!-- Import/Create Dialog -->
    <v-dialog v-model="importDialog" max-width="500px">
      <v-card>
        <v-card-title>
          <span class="text-h5">{{ isCreateMode ? 'Create Secret' : 'Import Secret' }}</span>
        </v-card-title>
        <v-card-text>
          <v-form ref="form" v-model="valid">
            <v-text-field
              v-model="newSecret.name"
              label="Secret Name"
              :rules="secretNameRules"
              required
              variant="outlined"
              class="mb-3"
            />
            <template v-if="isCreateMode">
              <v-text-field
                v-model="newSecret.key"
                label="Secret Value"
                type="text"
                :rules="[v => !!v || 'Secret Value is required']"
                required
                variant="outlined"
              />
            </template>
            <template v-else>
              <v-textarea
                v-model="newSecret.key"
                label="Encrypted secret value"
                :rules="encryptedSecretRules"
                required
                variant="outlined"
                rows="8"
                auto-grow
                style="font-family: monospace; font-size: 12px;"
              />
            </template>
          </v-form>
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn @click="closeImportDialog">Cancel</v-btn>
          <v-btn color="primary" @click="saveSecret" :disabled="!valid">
            {{ isCreateMode ? 'Create' : 'Save' }}
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Share Dialog -->
    <v-dialog v-model="shareDialog" max-width="600px">
      <v-card>
        <v-card-title>
          <span class="text-h5">Share "{{ selectedSecretForShare?.name }}"</span>
        </v-card-title>
        <v-card-text>
          <p class="mb-4">Select a user to share this secret with:</p>
          <v-text-field
            v-model="userSearch"
            label="Search Users"
            prepend-inner-icon="mdi-magnify"
            variant="outlined"
            clearable
            class="mb-4"
          />
          
          <div v-if="filteredUsersForShare.length === 0" class="text-center py-4">
            <v-icon size="48" color="grey">mdi-account-plus</v-icon>
            <p class="mt-2 text-grey">No users found. Add users first.</p>
          </div>
          
          <div v-else class="max-height-300 overflow-y-auto">
            <v-radio-group v-model="selectedUserForShare" class="ma-0">
              <v-radio
                v-for="user in filteredUsersForShare"
                :key="user.id"
                :value="user.id"
                :label="user.name"
                hide-details
                class="ma-0 pa-1"
              />
            </v-radio-group>
          </div>
          <v-alert v-if="encryptError" type="error" variant="tonal" class="mt-4">{{ encryptError }}</v-alert>
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn @click="closeShareDialog">Cancel</v-btn>
          <v-btn color="primary" @click="shareSecret" :disabled="!selectedUserForShare">
            Share with {{ selectedUserForShare ? filteredUsersForShare.find(u => u.id === selectedUserForShare)?.name : 'user' }}
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Encrypted Message Dialog -->
    <v-dialog v-model="encryptedMessageDialog" max-width="700px">
      <v-card>
        <v-card-title>
          <span class="text-h5">Encrypted Message for Sharing</span>
        </v-card-title>
        <v-card-text>
          <v-textarea
            v-model="encryptedMessage"
            label="Encrypted Message"
            readonly
            rows="8"
            variant="outlined"
            style="font-family: monospace; font-size: 12px;"
          />
          <div class="d-flex justify-end mt-2">
            <v-btn color="primary" prepend-icon="mdi-content-copy" @click="copyEncryptedMessage">
              Copy message
            </v-btn>
          </div>
          <div v-if="copySuccess" class="text-success text-caption mt-2 text-end">
            Copied!
          </div>
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn @click="closeEncryptedMessageDialog">Close</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Secret Details Dialog -->
    <v-dialog v-model="secretDetailsDialog" max-width="600px">
      <v-card>
        <v-card-title class="d-flex justify-space-between align-center">
          <span class="text-h5">Secret Details</span>
          <v-btn
            v-if="!isEditingSecret"
            icon
            size="small"
            variant="text"
            @click="startEditingSecret"
          >
            <v-icon>mdi-pencil</v-icon>
            <v-tooltip activator="parent">Edit Secret</v-tooltip>
          </v-btn>
        </v-card-title>
        <v-card-text v-if="selectedSecretDetails">
          <v-text-field
            v-model="editableSecret.name"
            label="Secret Name"
            :readonly="!isEditingSecret"
            variant="outlined"
            class="mb-3"
          />
          <v-text-field
            v-model="editableSecret.key"
            label="Secret Value"
            :type="showSecretKey ? 'text' : 'password'"
            :readonly="!isEditingSecret"
            variant="outlined"
            class="mb-3"
          >
            <template v-slot:append-inner>
              <v-btn
                icon
                size="small"
                variant="text"
                @click="toggleSecretVisibility"
              >
                <font-awesome-icon 
                  :icon="showSecretKey ? 'eye-slash' : 'eye'" 
                  class="text-grey-darken-1"
                />
              </v-btn>
            </template>
          </v-text-field>
          
          <!-- Timestamps -->
          <div class="d-flex justify-space-between text-caption text-grey">
            <div>
              <v-icon size="14" class="mr-1">mdi-calendar-plus</v-icon>
              Created: {{ formatDate(selectedSecretDetails.created_at) }}
            </div>
            <div>
              <v-icon size="14" class="mr-1">mdi-calendar-edit</v-icon>
              Updated: {{ formatDate(selectedSecretDetails.updated_at) }}
            </div>
          </div>
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <template v-if="isEditingSecret">
            <v-btn @click="cancelEditingSecret">Cancel</v-btn>
            <v-btn color="primary" @click="saveSecretChanges">Save Changes</v-btn>
          </template>
          <template v-else>
            <v-btn @click="closeSecretDetails">Close</v-btn>
          </template>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Confirmation Delete Dialog -->
    <v-dialog v-model="confirmDeleteDialog" max-width="400px">
      <v-card>
        <v-card-title class="text-error">
          <v-icon class="mr-2">mdi-alert</v-icon>
          Confirm Delete
        </v-card-title>
        <v-card-text v-if="secretToDelete">
          Are you sure you want to delete secret <strong>"{{ secretToDelete.name }}"</strong>?
          <br><br>
          <v-alert type="warning" variant="tonal">
            This action cannot be undone.
          </v-alert>
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn @click="cancelDelete">Cancel</v-btn>
          <v-btn color="error" @click="deleteSecret">Delete</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Password Prompt Dialog -->
    <v-dialog v-model="passwordPromptDialog" max-width="400px" persistent>
      <v-card>
        <v-card-title>
          <v-icon class="mr-2">mdi-lock</v-icon>
          Enter Master Password
        </v-card-title>
        <v-card-text>
          <p class="mb-4">
            {{ shareDialog ? 'Please enter your master password to share the secret.' : 'Please enter your master password to view the secret.' }}
          </p>
          <v-alert
            v-if="passwordErrorMessage"
            type="error"
            variant="tonal"
            class="mb-4"
            :text="passwordErrorMessage"
          />
          <v-form ref="passwordForm" v-model="passwordPromptValid">
            <v-text-field
              v-model="masterPassword"
              label="Master Password"
              type="password"
              :rules="masterPasswordRules"
              variant="outlined"
              autofocus
              @keyup.enter="handlePasswordDialogVerify"
              @input="passwordErrorMessage = ''"
            />
          </v-form>
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn @click="cancelPasswordPrompt" :disabled="isEncrypting || isValidatingPassword">
            Cancel
          </v-btn>
          <v-btn 
            color="primary" 
            @click="handlePasswordDialogVerify" 
            :disabled="!passwordPromptValid || isEncrypting || isValidatingPassword"
            :loading="isEncrypting || isValidatingPassword"
          >
            Verify{{ shareDialog ? ' & Share' : '' }}
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </v-container>
</template>

<style scoped>
.sticky-search {
  position: sticky;
  top: 0;
  z-index: 10;
}

.secret-card {
  cursor: pointer;
  transition: all 0.2s ease;
}

.secret-card:hover {
  transform: translateY(-2px);
}

.secrets-list {
  max-height: calc(100vh - 200px);
  overflow-y: auto;
}

.max-height-300 {
  max-height: 300px;
}

.gap-2 {
  gap: 8px;
}

.action-buttons {
  min-width: 140px;
  flex-shrink: 0;
}

.action-btn {
  opacity: 0.9;
  transition: all 0.2s ease;
  font-size: 11px;
  font-weight: 500;
  min-width: 60px;
  height: 28px;
  padding: 0 12px;
  display: flex !important;
  align-items: center !important;
  justify-content: center !important;
  text-align: center !important;
}

.action-btn .v-btn__content {
  display: flex !important;
  align-items: center !important;
  justify-content: center !important;
  width: 100% !important;
  text-align: center !important;
}

.secret-card:hover .action-btn {
  opacity: 1;
  transform: scale(1.05);
}
</style>