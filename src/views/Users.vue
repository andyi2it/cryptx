<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { addUser, getUsers, initDatabase } from '../lib/database';
import { getDatabase } from '../lib/database';

const search = ref('');
const users = ref<Array<{ id: number; name: string; public_key: string }>>([]);

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
const selectedUser = ref<{ id: number; name: string; public_key: string } | null>(null);

// Add confirmation dialog state
const confirmDeleteDialog = ref(false);
const userToDelete = ref<{ id: number; name: string; public_key: string } | null>(null);

const loadUsers = async () => {
  try {
    users.value = await getUsers();
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
      await addUser(newUser.value.name, newUser.value.publicKey);
      await loadUsers(); // Refresh the list
      closeAddUserDialog();
    } catch (error) {
      console.error('Failed to save user:', error);
      alert('Failed to save user: ' + error);
    }
  }
};

const openUserDetails = (user: { id: number; name: string; public_key: string }) => {
  selectedUser.value = user;
  userDetailsDialog.value = true;
};

const closeUserDetails = () => {
  userDetailsDialog.value = false;
  selectedUser.value = null;
};

// Update deleteUser function
const confirmDeleteUser = (user: { id: number; name: string; public_key: string }) => {
  userToDelete.value = user;
  confirmDeleteDialog.value = true;
};

const deleteUser = async () => {
  if (userToDelete.value) {
    try {
      const database = await getDatabase();
      await database.execute('DELETE FROM users WHERE id = ?', [userToDelete.value.id]);
      await loadUsers(); // Refresh the list
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
  <v-container>
    <v-text-field v-model="search" label="Search Users"></v-text-field>
    <v-list>
      <v-list-item 
        v-for="user in filteredUsers" 
        :key="user.id"
        @click="openUserDetails(user)"
        style="cursor: pointer"
      >
        <v-list-item-title>{{ user.name }}</v-list-item-title>
        <template v-slot:append>
          <v-btn
            text
            color="error"
            size="small"
            @click.stop="confirmDeleteUser(user)"
          >
            Delete
          </v-btn>
        </template>
      </v-list-item>
    </v-list>

    <!-- Add User Button -->
    <v-btn
      color="primary"
      size="large"
      position="fixed"
      location="bottom end"
      class="ma-4"
      rounded
      @click="openAddUserDialog"
    >
      Add User
    </v-btn>

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
            ></v-text-field>
            <v-textarea
              v-model="newUser.publicKey"
              label="Public Key"
              :rules="[v => !!v || 'Public key is required']"
              required
            ></v-textarea>
          </v-form>
        </v-card-text>
        <v-card-actions>
          <v-spacer></v-spacer>
          <v-btn color="blue darken-1" text @click="closeAddUserDialog">
            Cancel
          </v-btn>
          <v-btn color="blue darken-1" text @click="saveUser" :disabled="!valid">
            Save
          </v-btn>
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
          <v-row>
            <v-col cols="12">
              <v-text-field
                :model-value="selectedUser.name"
                label="User Name"
                readonly
              ></v-text-field>
            </v-col>
            <v-col cols="12">
              <v-textarea
                :model-value="selectedUser.public_key"
                label="Public Key"
                readonly
                rows="8"
              ></v-textarea>
            </v-col>
          </v-row>
        </v-card-text>
        <v-card-actions>
          <v-spacer></v-spacer>
          <v-btn color="blue darken-1" text @click="closeUserDetails">
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
        <v-card-text v-if="userToDelete">
          Are you sure you want to delete user "{{ userToDelete.name }}"?
        </v-card-text>
        <v-card-actions>
          <v-spacer></v-spacer>
          <v-btn color="blue darken-1" text @click="cancelDelete">
            Cancel
          </v-btn>
          <v-btn color="red darken-1" text @click="deleteUser">
            Delete
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </v-container>
</template>
