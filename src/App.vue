<template>
  <v-app :theme="currentTheme">
    <!-- Loading state -->
    <div v-if="isLoading" class="d-flex justify-center align-center" style="height: 100vh;">
      <v-progress-circular indeterminate color="primary" size="64">
        <template v-slot:default>Loading...</template>
      </v-progress-circular>
    </div>
    
    <!-- Login Dialog -->
    <LoginDialog 
      v-if="!isLoading"
      v-model="showLoginDialog"
      @login="handleLogin"
      @close="handleLoginClose"
    />
    
    <!-- Main Application Content - only show when logged in -->
    <template v-if="!isLoading && isLoggedIn">
      <v-navigation-drawer v-model="drawer" app width="280">
        <v-list nav>
          <v-list-item
            v-for="item in items"
            :key="item.title"
            @click="navigateTo(item.route)"
            :class="{ 'v-list-item--active': route.path === item.route }"
            :prepend-icon="item.icon"
          >
            <v-list-item-title :class="{ 'font-weight-bold': route.path === item.route }">
              {{ item.title }}
            </v-list-item-title>
          </v-list-item>
        </v-list>
      </v-navigation-drawer>

      <v-app-bar app elevation="2">
        <v-app-bar-nav-icon @click="drawer = !drawer" />
        <v-app-bar-title>{{ pageTitle }}</v-app-bar-title>
        <v-spacer />
        
        <!-- Show My Public Key Button - only visible on Users route -->
        <v-btn
          v-if="route.path === '/users'"
          color="primary"
          prepend-icon="mdi-key"
          @click="showMyPublicKey"
          variant="elevated"
          size="x-small"
          class="mr-4 d-flex justify-center align-center"
          style="min-width: 120px; font-size: 11px;"
        >
          Show My Public Key
        </v-btn>
        
        <!-- Dark mode switch -->
        <div class="d-flex align-center mr-4">
          <v-icon class="mr-2" :color="isDark ? 'grey' : 'orange'">
            mdi-weather-sunny
          </v-icon>
          <v-switch
            v-model="isDark"
            @change="toggleTheme"
            hide-details
            density="compact"
            color="primary"
          >
            <v-tooltip activator="parent">
              {{ isDark ? 'Switch to Light Mode' : 'Switch to Dark Mode' }}
            </v-tooltip>
          </v-switch>
          <v-icon class="ml-2" :color="isDark ? 'blue' : 'grey'">
            mdi-weather-night
          </v-icon>
        </div>
      </v-app-bar>

      <v-main>
        <router-view />
      </v-main>
      
      <!-- My Public Key Dialog -->
      <v-dialog v-model="myPublicKeyDialog" max-width="700px">
        <v-card>
          <v-card-title class="d-flex justify-space-between align-center">
            <span class="text-h5">My Public Key</span>
            <!-- <v-btn
              icon
              size="small"
              variant="text"
              @click="closeMyPublicKeyDialog"
            >
              <v-icon>mdi-close</v-icon>
            </v-btn> -->
          </v-card-title>
          
          <v-card-text>
            <div v-if="isLoadingPublicKey" class="d-flex justify-center py-6">
              <v-progress-circular indeterminate color="primary">
                Loading...
              </v-progress-circular>
            </div>
            
            <div v-else-if="publicKeyError" class="text-center py-6">
              <v-alert type="error" variant="tonal" class="mb-4">
                {{ publicKeyError }}
              </v-alert>
            </div>
            
            <div v-else>
              <p class="text-caption text-grey mb-3">
                Share this public key with others so they can encrypt messages for you.
              </p>
              
              <v-textarea
                v-model="myPublicKey"
                label="Public Key"
                readonly
                rows="12"
                variant="outlined"
                class="mb-3"
                style="font-family: monospace; font-size: 12px;"
              />
              
              <div class="d-flex justify-end">
                <v-btn
                  color="primary"
                  prepend-icon="mdi-content-copy"
                  @click="copyPublicKey"
                  variant="elevated"
                >
                  <font-awesome-icon icon="fa-solid fa-copy" class="mr-2" />
                  Copy Public Key
                </v-btn>
              </div>
            </div>
          </v-card-text>
          
          <v-card-actions v-if="!isLoadingPublicKey">
            <v-spacer />
            <v-btn @click="closeMyPublicKeyDialog">Close</v-btn>
          </v-card-actions>
        </v-card>
      </v-dialog>
    </template>
  </v-app>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { useRouter, useRoute } from 'vue-router';
import { useTheme } from 'vuetify';
import Initialize from './Initialize.vue';
import LoginDialog from './components/LoginDialog.vue';
import { checkKeyFileExists } from './helpers/init';
import { hasStoredCredentials } from './helpers/auth';
import { invoke } from '@tauri-apps/api/core';

const theme = useTheme();
const isDark = ref(false);
const currentTheme = computed(() => isDark.value ? 'dark' : 'light');

const drawer = ref(true);
const items = [
  { title: 'Secrets', icon: 'mdi-shield-key', route: '/secrets' },
  { title: 'Users', icon: 'mdi-account-group', route: '/users' },
];

const router = useRouter();
const route = useRoute();
const showInitializeModal = ref(false);

// Login state
const isLoggedIn = ref(false);
const showLoginDialog = ref(false);
const isLoading = ref(true);

// My Public Key dialog state
const myPublicKeyDialog = ref(false);
const myPublicKey = ref('');
const isLoadingPublicKey = ref(false);
const publicKeyError = ref('');

function navigateTo(routePath: string) {
  console.log("navigating to route", routePath);
  router.push(routePath);
}

function toggleTheme() {
  theme.global.name.value = currentTheme.value;
  localStorage.setItem('cryptx-theme', currentTheme.value);
}

const handleLogin = async (credentials: { email: string; password: string }) => {
  console.log('User logged in with email:', credentials.email);
  
  // Set logged in state immediately
  isLoggedIn.value = true;
  showLoginDialog.value = false;
  
  // Check if we need to initialize (generate keys)
  try {
    const keyFileExists = await checkKeyFileExists();
    if (!keyFileExists) {
      console.log('Keys not found, but user is logged in - they should have been generated during setup');
    }
    
    // Navigate to default route
    if (route.path === '/' || route.path === '') {
      router.push('/secrets');
    }
  } catch (error) {
    console.error('Error checking key files after login:', error);
  }
};

const handleLoginClose = () => {
  // Allow user to close login and access app without authentication
  isLoggedIn.value = true;
  showLoginDialog.value = false;
  console.log('Login dialog closed, allowing access to app');
  
  // Navigate to default route
  if (route.path === '/' || route.path === '') {
    router.push('/secrets');
  }
};

onMounted(async () => {
  console.log('App mounting...');
  
  // Load saved theme preference
  const savedTheme = localStorage.getItem('cryptx-theme');
  if (savedTheme) {
    isDark.value = savedTheme === 'dark';
    theme.global.name.value = savedTheme;
  }

  try {
    // Check if user has stored credentials
    const hasCredentials = await hasStoredCredentials();
    console.log('Has stored credentials:', hasCredentials);
    
    if (hasCredentials) {
      // Show login dialog to validate credentials
      showLoginDialog.value = true;
      isLoggedIn.value = false;
    } else {
      // No credentials, show setup
      showLoginDialog.value = true;
      isLoggedIn.value = false;
    }
  } catch (error) {
    console.error('Failed to check stored credentials:', error);
    showLoginDialog.value = true;
    isLoggedIn.value = false;
  } finally {
    isLoading.value = false;
  }
});

const pageTitle = computed(() => {
  switch (route.path) {
    case '/secrets':
      return 'Secrets Manager'
    case '/users':
      return 'User Management'
    default:
      return 'CryptX'
  }
})

const showMyPublicKey = async () => {
  myPublicKeyDialog.value = true;
  isLoadingPublicKey.value = true;
  publicKeyError.value = '';
  
  try {
    // Get app data directory and read public key
    const appDataDir = await invoke('get_app_data_dir') as string;
    const publicKeyPath = await invoke('join_path', {
      base: appDataDir,
      segment: 'public_key.asc'
    }) as string;

    // Check if public key file exists
    const fileExists = await invoke('file_exists', { path: publicKeyPath }) as boolean;
    
    if (fileExists) {
      const publicKeyContent = await invoke('read_text_file', { path: publicKeyPath }) as string;
      myPublicKey.value = publicKeyContent;
    } else {
      publicKeyError.value = 'Public key file not found. Please set up your account first.';
    }
  } catch (error) {
    console.error('Failed to load public key:', error);
    publicKeyError.value = 'Failed to load public key. Please try again.';
  } finally {
    isLoadingPublicKey.value = false;
  }
};

const copyPublicKey = async () => {
  try {
    await navigator.clipboard.writeText(myPublicKey.value);
    console.log('Public key copied to clipboard');
  } catch (error) {
    console.error('Failed to copy public key:', error);
  }
};

const closeMyPublicKeyDialog = () => {
  myPublicKeyDialog.value = false;
  myPublicKey.value = '';
  publicKeyError.value = '';
};
</script>

<style scoped>
.container {
  margin: 0 auto;
  padding-top: 10vh;
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  max-width: 400px;
}

h1 {
  margin-bottom: 20px;
}

.form {
  display: flex;
  flex-direction: column;
  width: 100%;
}

input {
  margin-bottom: 10px;
  padding: 10px;
  font-size: 1em;
  border: 1px solid #ccc;
  border-radius: 4px;
}

button {
  padding: 10px;
  font-size: 1em;
  color: #fff;
  background-color: #0073b1;
  border: none;
  border-radius: 4px;
  cursor: pointer;
}

button:hover {
  background-color: #005582;
}

.divider {
  margin: 20px 0;
  font-size: 1em;
  color: #666;
}

.facebook-button {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 10px;
  font-size: 1em;
  color: #fff;
  background-color: #3b5998;
  border: none;
  border-radius: 4px;
  cursor: pointer;
}

.facebook-button img {
  margin-right: 10px;
}

.facebook-button:hover {
  background-color: #2d4373;
}

a {
  color: #0073b1;
  text-decoration: none;
}

a:hover {
  text-decoration: underline;
}
</style>