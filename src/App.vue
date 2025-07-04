<template>
  <v-app :vuetify="vuetify">
    <v-navigation-drawer v-model="drawer" app>
      <v-list>
        <v-list-item
          v-for="item in items"
          :key="item.title"
          @click="navigateTo(item.route)"
          :active="route.path === item.route"
        >
          <v-list-item-title>{{ item.title }}</v-list-item-title>
        </v-list-item>
      </v-list>
    </v-navigation-drawer>

    <v-app-bar app>
      <v-toolbar-title>{{ pageTitle }}</v-toolbar-title>
    </v-app-bar>

    <v-main>
      <v-container>
        <router-view></router-view>
      </v-container>
    </v-main>
    <v-dialog v-model="showInitializeModal" persistent max-width="600px">
      <Initialize />
    </v-dialog>
  </v-app>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { useRouter, useRoute } from 'vue-router';
import { createVuetify } from 'vuetify';
import 'vuetify/styles';
import { VApp, VAppBar, VNavigationDrawer, VList, VListItem, VMain, VContainer, VTextField, VDialog } from 'vuetify/components';
import Initialize from './Initialize.vue';
import { checkKeyFileExists } from './helpers/init';

const vuetify = createVuetify();

const drawer = ref(true);
const items = [
  { title: 'Secrets', icon: 'mdi-lock', route: '/secrets' },
  { title: 'Users', icon: 'mdi-account', route: '/users' },
];

const router = useRouter();
const route = useRoute();
const showInitializeModal = ref(false);

function navigateTo(route) {
  console.log("navigating to route", route);
  router.push(route);
}

onMounted(async () => {
  const keyFileExists = await checkKeyFileExists();
  if (!keyFileExists) {
    showInitializeModal.value = true;
  }
});

const pageTitle = computed(() => {
  switch (route.path) {
    case '/secrets':
      return 'Secrets'
    case '/users':
      return 'Users'
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