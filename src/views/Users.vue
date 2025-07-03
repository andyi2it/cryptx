<script setup lang="ts">
import { ref, computed } from 'vue';
import { useUsers } from '../composables/useUsers';
const search = ref('');
const { users, addUser } = useUsers();

const filteredUsers = computed(() => {
  return users.value.filter(user => user.toLowerCase().includes(search.value.toLowerCase()));
});

// Add user dialog state
const addUserDialog = ref(false);
const valid = ref(false);
const form = ref();
const newUser = ref({
  name: '',
  publicKey: ''
});

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

const saveUser = () => {
  if (valid.value) {
    addUser(newUser.value.name);
    closeAddUserDialog();
  }
};
</script>

<template>
  <v-container>
    <v-text-field v-model="search" label="Search Users"></v-text-field>
    <v-list>
      <v-list-item v-for="user in filteredUsers" :key="user">
        <v-list-item-title>{{ user }}</v-list-item-title>
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
  </v-container>
</template>
