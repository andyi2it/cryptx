<template>
  <v-container>
    <v-text-field v-model="search" label="Search Secrets"></v-text-field>
    <v-list>
      <v-list-item v-for="secret in filteredSecrets" :key="secret">
        <v-list-item-title>{{ secret }}</v-list-item-title>
        <template v-slot:append>
          <v-btn
            text
            color="primary"
            size="small"
            @click="openShareDialog(secret)"
          >
            Share
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
              :key="user"
              @click="toggleUserSelection(user)"
            >
              <template v-slot:prepend>
                <v-checkbox
                  :model-value="selectedUsers.includes(user)"
                  @click.stop="toggleUserSelection(user)"
                  hide-details
                ></v-checkbox>
              </template>
              <v-list-item-title>{{ user }}</v-list-item-title>
            </v-list-item>
          </v-list>
          <div v-if="selectedUsers.length > 0" class="mt-4">
            <v-chip-group>
              <v-chip
                v-for="user in selectedUsers"
                :key="user"
                closable
                @click:close="removeUserFromSelection(user)"
              >
                {{ user }}
              </v-chip>
            </v-chip-group>
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
            @click="shareToSelectedUsers"
            :disabled="selectedUsers.length === 0"
          >
            Share
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </v-container>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';

const search = ref('');
const secrets = ref([
  'Secret 1',
  'Secret 2',
  'Secret 3'
]);

const filteredSecrets = computed(() => {
  return secrets.value.filter(secret => secret.toLowerCase().includes(search.value.toLowerCase()));
});

// Import dialog state
const importDialog = ref(false);
const valid = ref(false);
const form = ref();
const newSecret = ref({
  name: '',
  key: ''
});

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

const saveSecret = () => {
  if (valid.value) {
    secrets.value.push(newSecret.value.name);
    closeImportDialog();
  }
};

// Share dialog state
const shareDialog = ref(false);
const selectedSecret = ref('');
const userSearch = ref('');
const selectedUsers = ref<string[]>([]);
const users = ref([
  'User 1',
  'User 2', 
  'User 3',
  'Alice',
  'Bob',
  'Charlie'
]);

const filteredUsersForShare = computed(() => {
  return users.value.filter(user => 
    user.toLowerCase().includes(userSearch.value.toLowerCase())
  );
});

const openShareDialog = (secret: string) => {
  selectedSecret.value = secret;
  shareDialog.value = true;
  userSearch.value = '';
  selectedUsers.value = [];
};

const closeShareDialog = () => {
  shareDialog.value = false;
  selectedSecret.value = '';
  userSearch.value = '';
  selectedUsers.value = [];
};

const shareToSelectedUsers = () => {
  // Handle sharing logic here
  console.log(`Sharing "${selectedSecret.value}" to`, selectedUsers.value);
  closeShareDialog();
};

const toggleUserSelection = (user: string) => {
  const index = selectedUsers.value.indexOf(user);
  if (index > -1) {
    selectedUsers.value.splice(index, 1);
  } else {
    selectedUsers.value.push(user);
  }
};

const removeUserFromSelection = (user: string) => {
  const index = selectedUsers.value.indexOf(user);
  if (index > -1) {
    selectedUsers.value.splice(index, 1);
  }
};
</script>