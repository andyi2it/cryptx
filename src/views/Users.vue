<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { addUser, getUsers, initDatabase, deleteUser as dbDeleteUser } from '../lib/database';
import { invoke } from '@tauri-apps/api/core';

const search = ref('');
const users = ref<Array<{ id: number; name: string; email: string; public_key: string }>>([]);

const filteredUsers = computed(() => {
  return users.value.filter(user => 
    user.name.toLowerCase().includes(search.value.toLowerCase())
  );
});

// Add user dialog state
const addUserDialog = ref(false);
const valid = ref(false);
const form = ref();
const newUser = ref({
  name: '',
  publicKey: ''
});

// User details dialog state
const userDetailsDialog = ref(false);
const selectedUser = ref<{ id: number; name: string; email: string; public_key: string } | null>(null);

// Confirmation dialog state
const confirmDeleteDialog = ref(false);
const userToDelete = ref<{ id: number; name: string; email: string; public_key: string } | null>(null);

const loadUsers = async () => {
  try {
    users.value = await getUsers();
    // loop users.value and print
    users.value.forEach(user => {
      console.log(`User: ${user.name}, Email: ${user.email}`, user.public_key);
    });
  } catch (error) {
    console.error('Failed to load users:', error);
  }
};

const openAddUserDialog = () => {
  addUserDialog.value = true;
};

const closeAddUserDialog = () => {
  addUserDialog.value = false;
  newUser.value = { name: '', publicKey: '' };
  if (form.value) {
    form.value.reset();
  }
};

const saveUser = async () => {
  if (valid.value) {
    try {
      // Extract email from public key using backend
      const extracted_email_id: string = await invoke('get_email_ids_from_public_key', { pubkey: newUser.value.publicKey });
      console.debug('Extracted Email ID:', extracted_email_id);
      await addUser(newUser.value.name, extracted_email_id, newUser.value.publicKey);
      await loadUsers(); // Refresh the list
      closeAddUserDialog();
    } catch (error) {
      console.error('Failed to save user:', error);
      alert('Failed to save user: ' + error);
    }
  }
};

const openUserDetails = (user: { id: number; name: string; email: string; public_key: string }) => {
  selectedUser.value = user;
  userDetailsDialog.value = true;
};

const closeUserDetails = () => {
  userDetailsDialog.value = false;
  selectedUser.value = null;
};

// Update deleteUser function
const confirmDeleteUser = (user: { id: number; name: string; email: string; public_key: string }) => {
  userToDelete.value = user;
  confirmDeleteDialog.value = true;
};

const deleteUser = async () => {
  if (userToDelete.value) {
    try {
      await dbDeleteUser(userToDelete.value.id);
      await loadUsers();
      confirmDeleteDialog.value = false;
      userToDelete.value = null;
    } catch (error) {
      console.error('Failed to delete user:', error);
      alert('Failed to delete user: ' + error);
    }
  }
};

const cancelDelete = () => {
  confirmDeleteDialog.value = false;
  userToDelete.value = null;
};

onMounted(async () => {
  await initDatabase();
  await loadUsers();
});
</script>

<template>
  <v-container fluid class="pa-0">
    <!-- Sticky Search Bar -->
    <v-card class="sticky-search ma-4 mb-2" elevation="2">
      <v-card-text class="pb-2">
        <v-text-field
          v-model="search"
          placeholder="Search users..."
          prepend-inner-icon="mdi-magnify"
          variant="outlined"
          hide-details
          clearable
          density="comfortable"
        />
      </v-card-text>
    </v-card>

    <!-- Add User Button -->
    <div class="px-4 pb-2">
      <v-btn
        color="primary"
        prepend-icon="mdi-account-plus"
        @click="openAddUserDialog"
        variant="elevated"
      >
        Add User
      </v-btn>
    </div>

    <!-- Users List -->
    <div class="users-list px-4">
      <div v-if="filteredUsers.length === 0" class="text-center py-8">
        <v-icon size="64" color="grey-lighten-1">mdi-account-outline</v-icon>
        <h3 class="mt-4 text-grey">No users found</h3>
        <p class="text-grey">Add your first user to get started</p>
      </div>

      <v-card
        v-for="user in filteredUsers"
        :key="user.id"
        class="mb-4 user-card"
        elevation="2"
        hover
      >
        <v-card-text class="pa-4">
          <div class="d-flex justify-space-between align-center">
            <div class="d-flex align-center flex-grow-1" @click="openUserDetails(user)" style="cursor: pointer;">
              <div class="user-icon-container mr-3">
                <font-awesome-icon 
                  :icon="['fas', 'user']" 
                  class="user-icon"
                />
              </div>
              <div>
                <h4 class="text-h6">{{ user.name }}</h4>
                <p class="text-caption text-grey">Public key configured</p>
              </div>
            </div>

            <!-- Action button - always visible with proper styling -->
            <div class="action-buttons d-flex align-center ml-4">
              <v-btn
                size="small"
                variant="elevated"
                color="error"
                @click.stop="confirmDeleteUser(user)"
                class="delete-btn"
              >
                DELETE
              </v-btn>
            </div>
          </div>
        </v-card-text>
      </v-card>
    </div>

    <!-- Add User Dialog -->
    <v-dialog v-model="addUserDialog" max-width="500px">
      <v-card>
        <v-card-title>
          <span class="text-h5">Add User</span>
        </v-card-title>
        <v-card-text>
          <v-form ref="form" v-model="valid">
            <v-text-field
              v-model="newUser.name"
              label="User Name"
              :rules="[v => !!v || 'User name is required']"
              required
              variant="outlined"
              class="mb-3"
            />
            <v-textarea
              v-model="newUser.publicKey"
              label="Public Key"
              :rules="[v => !!v || 'Public key is required']"
              required
              variant="outlined"
              rows="4"
            />
          </v-form>
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn @click="closeAddUserDialog">Cancel</v-btn>
          <v-btn color="primary" @click="saveUser" :disabled="!valid">Save</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- User Details Dialog -->
    <v-dialog v-model="userDetailsDialog" max-width="600px">
      <v-card>
        <v-card-title>
          <span class="text-h5">User Details</span>
        </v-card-title>
        <v-card-text v-if="selectedUser">
          <v-text-field
            :model-value="selectedUser.name"
            label="User Name"
            readonly
            variant="outlined"
            class="mb-3"
          />
          <v-text-field
            :model-value="selectedUser.email"
            label="Email"
            readonly
            variant="outlined"
            class="mb-3"
          />
          <v-textarea
            :model-value="selectedUser.public_key"
            label="Public Key"
            readonly
            rows="8"
            variant="outlined"
          />
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn @click="closeUserDetails">Close</v-btn>
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
        <v-card-text v-if="userToDelete">
          Are you sure you want to delete user <strong>"{{ userToDelete.name }}"</strong>?
          <br><br>
          <v-alert type="warning" variant="tonal">
            This action cannot be undone.
          </v-alert>
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn @click="cancelDelete">Cancel</v-btn>
          <v-btn color="error" @click="deleteUser">Delete</v-btn>
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

.user-card {
  cursor: pointer;
  transition: all 0.2s ease;
}

.user-card:hover {
  transform: translateY(-2px);
}

.users-list {
  max-height: calc(100vh - 200px);
  overflow-y: auto;
}

.action-buttons {
  min-width: 80px;
  flex-shrink: 0;
}

.delete-btn {
  opacity: 0.9;
  transition: all 0.2s ease;
  font-size: 12px;
  min-width: 60px;
  display: flex !important;
  align-items: center !important;
  justify-content: center !important;
  text-align: center !important;
}

.delete-btn .v-btn__content {
  display: flex !important;
  align-items: center !important;
  justify-content: center !important;
  width: 100% !important;
}

.user-card:hover .delete-btn {
  opacity: 1;
  transform: scale(1.05);
}

.user-icon-container {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: var(--v-theme-primary);
  border-radius: 50%;
}

.user-icon {
  color: white;
  font-size: 18px;
}
</style>
