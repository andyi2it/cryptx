<template>
  <v-container>
    <v-text-field v-model="search" label="Search Secrets"></v-text-field>
    <v-list>
      <v-list-item v-for="secret in filteredSecrets" :key="secret">
        <v-list-item-title>{{ secret }}</v-list-item-title>
      </v-list-item>
    </v-list>
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
</script>