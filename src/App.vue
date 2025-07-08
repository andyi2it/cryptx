<template>
  <v-app :theme="currentTheme">
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
    
    <v-dialog v-model="showInitializeModal" persistent max-width="600px">
      <Initialize @close="showInitializeModal = false" />
    </v-dialog>
  </v-app>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { useRouter, useRoute } from 'vue-router';
import { useTheme } from 'vuetify';
import Initialize from './Initialize.vue';
import { checkKeyFileExists } from './helpers/init';

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

function navigateTo(routePath: string) {
  console.log("navigating to route", routePath);
  router.push(routePath);
}

function toggleTheme() {
  theme.global.name.value = currentTheme.value;
  localStorage.setItem('cryptx-theme', currentTheme.value);
}

onMounted(async () => {
  // Load saved theme preference
  const savedTheme = localStorage.getItem('cryptx-theme');
  if (savedTheme) {
    isDark.value = savedTheme === 'dark';
    theme.global.name.value = savedTheme;
  }

  const keyFileExists = await checkKeyFileExists();
  if (!keyFileExists) {
    showInitializeModal.value = true;
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