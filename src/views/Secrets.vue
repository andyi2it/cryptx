<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { addSecret, getSecrets, getUsers, initDatabase } from '../lib/database';
import { getDatabase } from '../lib/database';

const search = ref('');
const secrets = ref<Array<{ id: number; name: string; key: string }>>([]);

const filteredSecrets = computed(() => {
  return secrets.value.filter(secret => 
    secret.name.toLowerCase().includes(search.value.toLowerCase())
  );
});

// Import dialog state
const importDialog = ref(false);
const valid = ref(false);
const form = ref();
const newSecret = ref({
  name: '',
  key: ''
});

// Share dialog state
const shareDialog = ref(false);
const selectedSecret = ref('');
const userSearch = ref('');
const selectedUser = ref<{ id: number; name: string; public_key: string } | null>(null);
const users = ref<Array<{ id: number; name: string; public_key: string }>>([]);

const filteredUsersForShare = computed(() => {
  return users.value.filter(user => 
    user.name.toLowerCase().includes(userSearch.value.toLowerCase())
  );
});

// Secret details dialog state
const secretDetailsDialog = ref(false);
const selectedSecretDetails = ref<{ id: number; name: string; key: string } | null>(null);
const showSecretKey = ref(false);

// Add confirmation dialog state
const confirmDeleteDialog = ref(false);
const secretToDelete = ref<{ id: number; name: string; key: string } | null>(null);

const loadSecrets = async () => {
  try {
    secrets.value = await getSecrets();
  } catch (error) {
    console.error('Failed to load secrets:', error);
  }
};

const loadUsers = async () => {
  try {
    users.value = await getUsers();
  } catch (error) {
    console.error('Failed to load users:', error);
  }
};

const openImportDialog = () => {
  importDialog.value = true;
};

const closeImportDialog = () => {
  importDialog.value = false;
  newSecret.value = { name: '', key: '' };
  if (form.value) {
    form.value.reset();
  }
};

const saveSecret = async () => {
  if (valid.value) {
    try {
      await addSecret(newSecret.value.name, newSecret.value.key);
      await loadSecrets(); // Refresh the list
      closeImportDialog();
    } catch (error) {
      console.error('Failed to save secret:', error);
      alert('Failed to save secret: ' + error);
    }
  }
};

const openShareDialog = (secret: string) => {
  selectedSecret.value = secret;
  shareDialog.value = true;
  userSearch.value = '';
  selectedUser.value = null;
};

const closeShareDialog = () => {
  shareDialog.value = false;
  selectedSecret.value = '';
  userSearch.value = '';
  selectedUser.value = null;
};

const shareToUser = (user: { id: number; name: string; public_key: string }) => {
  selectedUser.value = user;
  // Handle sharing logic here
  console.log(`Sharing "${selectedSecret.value}" to ${user.name}`);
  closeShareDialog();
};

const confirmDeleteSecret = (secret: { id: number; name: string; key: string }) => {
  secretToDelete.value = secret;
  confirmDeleteDialog.value = true;
};

const deleteSecret = async () => {
  if (secretToDelete.value) {
    try {
      const database = await getDatabase();
      await database.execute('DELETE FROM secrets WHERE id = ?', [secretToDelete.value.id]);
      await loadSecrets(); // Refresh the list
      confirmDeleteDialog.value = false;
      secretToDelete.value = null;
    } catch (error) {
      console.error('Failed to delete secret:', error);
      alert('Failed to delete secret: ' + error);
    }
  }
};

const cancelDelete = () => {
  confirmDeleteDialog.value = false;
  secretToDelete.value = null;
};

const openSecretDetails = (secret: { id: number; name: string; key: string }) => {
  selectedSecretDetails.value = secret;
  secretDetailsDialog.value = true;
};

const closeSecretDetails = () => {
  secretDetailsDialog.value = false;
  selectedSecretDetails.value = null;
  showSecretKey.value = false;
};

const toggleSecretVisibility = () => {
  showSecretKey.value = !showSecretKey.value;
};

onMounted(async () => {
  await initDatabase();
  await loadSecrets();
  await loadUsers();
});
</script>

<template>
  <v-container>
    <v-text-field v-model="search" label="Search Secrets"></v-text-field>
    <v-list>
      <v-list-item 
        v-for="secret in filteredSecrets" 
        :key="secret.id"
        @click="openSecretDetails(secret)"
        style="cursor: pointer"
      >
        <v-list-item-title>{{ secret.name }}</v-list-item-title>
        <template v-slot:append>
          <v-btn
            text
            color="primary"
            size="small"
            @click.stop="openShareDialog(secret.name)"
            class="mr-2"
          >
            Share
          </v-btn>
          <v-btn
            text
            color="error"
            size="small"
            @click.stop="confirmDeleteSecret(secret)"
          >
            Delete
          </v-btn>
        </template>
      </v-list-item>
    </v-list>

    <!-- Import Button -->
    <v-btn
      color="primary"
      size="large"
      position="fixed"
      location="bottom end"
      class="ma-4"
      rounded
      @click="openImportDialog"
    >
      Import
    </v-btn>

    <!-- Import Dialog -->
    <v-dialog v-model="importDialog" max-width="500px">
      <v-card>
        <v-card-title>
          <span class="text-h5">Import Secret</span>
        </v-card-title>
        <v-card-text>
          <v-form ref="form" v-model="valid">
            <v-text-field
              v-model="newSecret.name"
              label="Secret Name"
              :rules="[v => !!v || 'Secret name is required']"
              required
            ></v-text-field>
            <v-text-field
              v-model="newSecret.key"
              label="Secret Key"
              type="password"
              :rules="[v => !!v || 'Secret key is required']"
              required
            ></v-text-field>
          </v-form>
        </v-card-text>
        <v-card-actions>
          <v-spacer></v-spacer>
          <v-btn color="blue darken-1" text @click="closeImportDialog">
            Cancel
          </v-btn>
          <v-btn color="blue darken-1" text @click="saveSecret" :disabled="!valid">
            Save
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Share Dialog -->
    <v-dialog v-model="shareDialog" max-width="500px">
      <v-card>
        <v-card-title>
          <span class="text-h5">Share "{{ selectedSecret }}"</span>
        </v-card-title>
        <v-card-text>
          <v-text-field
            v-model="userSearch"
            label="Search Users"
            prepend-inner-icon="mdi-magnify"
            clearable
          ></v-text-field>
          <v-list>
            <v-list-item
              v-for="user in filteredUsersForShare"
              :key="user.id"
              @click="shareToUser(user)"
            >
              <v-list-item-title>{{ user.name }}</v-list-item-title>
            </v-list-item>
          </v-list>
          <div v-if="selectedUser" class="mt-4">
            <v-chip color="primary">
              Selected: {{ selectedUser.name }}
            </v-chip>
          </div>
        </v-card-text>
        <v-card-actions>
          <v-spacer></v-spacer>
          <v-btn color="blue darken-1" text @click="closeShareDialog">
            Cancel
          </v-btn>
          <v-btn 
            color="blue darken-1" 
            text 
            @click="shareToUser(selectedUser!)"
            :disabled="!selectedUser"
          >
            Share
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Secret Details Dialog -->
    <v-dialog v-model="secretDetailsDialog" max-width="600px">
      <v-card>
        <v-card-title>
          <span class="text-h5">Secret Details</span>
        </v-card-title>
        <v-card-text v-if="selectedSecretDetails">
          <v-row>
            <v-col cols="12">
              <v-text-field
                :model-value="selectedSecretDetails.name"
                label="Secret Name"
                readonly
              ></v-text-field>
            </v-col>
            <v-col cols="12">
              <v-text-field
                :model-value="selectedSecretDetails.key"
                label="Secret Key"
                :type="showSecretKey ? 'text' : 'password'"
                readonly
              >
                <template v-slot:append-inner>
                  <font-awesome-icon 
                    :icon="showSecretKey ? 'eye-slash' : 'eye'" 
                    @click="toggleSecretVisibility"
                    style="cursor: pointer; font-size: 16px;"
                  />
                </template>
              </v-text-field>
            </v-col>
          </v-row>
        </v-card-text>
        <v-card-actions>
          <v-spacer></v-spacer>
          <v-btn color="blue darken-1" text @click="closeSecretDetails">
            Close
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Confirmation Delete Dialog -->
    <v-dialog v-model="confirmDeleteDialog" max-width="400px">
      <v-card>
        <v-card-title>
          <span class="text-h5">Confirm Delete</span>
        </v-card-title>
        <v-card-text v-if="secretToDelete">
          Are you sure you want to delete secret "{{ secretToDelete.name }}"?
        </v-card-text>
        <v-card-actions>
          <v-spacer></v-spacer>
          <v-btn color="blue darken-1" text @click="cancelDelete">
            Cancel
          </v-btn>
          <v-btn color="red darken-1" text @click="deleteSecret">
            Delete
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </v-container>
</template>