<template>
  <!-- Initial setup is handled by LoginDialog now -->
</template>

<script setup lang="ts">
import { onMounted } from 'vue';

const emit = defineEmits<{
  'close': [];
  'setup-complete': [];
}>();

onMounted(() => {
  // Immediately emit setup-complete to close this component
  emit('setup-complete');
});
</script>

<style scoped>
/* Add any styles if needed */
</style>