<script setup lang="ts">
import { ref, computed } from 'vue';

const search = ref('');
const users = ref([
  'User 1',
  'User 2',
  'User 3'
]);

const filteredUsers = computed(() => {
  return users.value.filter(user => user.toLowerCase().includes(search.value.toLowerCase()));
});
</script>

<template>
  <v-container>
    <v-text-field v-model="search" label="Search Users"></v-text-field>
    <v-list>
      <v-list-item v-for="user in filteredUsers" :key="user">
        <v-list-item-title>{{ user }}</v-list-item-title>
      </v-list-item>
    </v-list>
  </v-container>
</template>
