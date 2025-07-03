<template>
  <v-card>
    <v-card-title>
      Initial Setup
      <v-spacer></v-spacer>
      <v-btn icon @click="$emit('close')" aria-label="Close">
        <v-icon>mdi-close</v-icon>
      </v-btn>
    </v-card-title>
    <v-card-text>
      <v-form>
        <v-text-field v-model="email" label="Email" required></v-text-field>
        <v-text-field v-model="password" label="Key Password - Do not use email password" type="password" required></v-text-field>
      </v-form>
    </v-card-text>
    <v-card-actions>
      <v-btn color="primary" @click="initialize">Save</v-btn>
    </v-card-actions>
  </v-card>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { init } from './helpers/init';

const email = ref('andrews.arokiam@ideas2it.com');
const password = ref('alphatango');

function initialize() {
  init(email.value, password.value);
}
</script>

<style scoped>
/* Add any styles if needed */
</style>