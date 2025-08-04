<template>
  <v-dialog v-model="show" max-width="400px" persistent>
    <v-card>
      <v-card-title class="text-center pa-6 position-relative">
        <v-btn
          size="small"
          variant="text"
          class="position-absolute"
          style="top: 16px; right: 16px;"
          @click="closeDialog"
        >
          Cancel
        </v-btn>
        <div class="d-flex flex-column align-center">
          <v-icon size="48" color="primary" class="mb-3">mdi-shield-key</v-icon>
          <h2 class="text-h5">{{ isRegistering ? 'Setup CryptX' : 'CryptX Login' }}</h2>
        </div>
      </v-card-title>
      
      <v-card-text class="pa-6">
        <v-form ref="form" v-model="valid" @submit.prevent="handleLogin">
          <v-text-field
            v-model="email"
            label="Email"
            type="email"
            :rules="emailRules"
            required
            variant="outlined"
            prepend-inner-icon="mdi-email"
            class="mb-3"
          />
          
          <v-text-field
            v-model="password"
            label="Master Password"
            type="password"
            :rules="passwordRules"
            required
            variant="outlined"
            prepend-inner-icon="mdi-lock"
            class="mb-2"
          />
          
          <v-text-field
            v-if="isRegistering"
            v-model="confirmPassword"
            label="Confirm Master Password"
            type="password"
            :rules="confirmPasswordRules"
            required
            variant="outlined"
            prepend-inner-icon="mdi-lock-check"
            class="mb-2"
          />
          
          <v-alert 
            v-if="loginError"
            type="error"
            variant="tonal"
            class="mb-3"
            density="compact"
          >
            {{ loginError }}
          </v-alert>
          
          <v-alert 
            type="info" 
            variant="tonal" 
            class="mb-4 text-caption"
            density="compact"
          >
            <strong>Security Note:</strong> {{ isRegistering ? 'Choose a strong master password. This will be used to secure your data.' : 'Enter your master password to access CryptX.' }}
          </v-alert>
          
          <v-btn
            type="submit"
            color="primary"
            block
            size="large"
            :loading="loading"
            :disabled="!valid"
            class="mt-2 mb-3"
          >
            {{ isRegistering ? 'Setup Account' : 'Login' }}
          </v-btn>
          
          <v-btn
            v-if="!isRegistering"
            @click="toggleMode"
            variant="text"
            block
            size="small"
          >
            First time? Setup new account
          </v-btn>
          
          <v-btn
            @click="handleTempInit"
            variant="outlined"
            color="secondary"
            block
            size="small"
            class="mt-2"
          >
            Temp Init (Dev)
          </v-btn>
        </v-form>
      </v-card-text>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { saveLoginCredentials, validateLoginCredentials, hasStoredCredentials, getStoredEmail } from '../helpers/auth';
import { init } from '../helpers/init';

const props = defineProps<{
  modelValue: boolean;
}>();

const emit = defineEmits<{
  'update:modelValue': [value: boolean];
  'login': [credentials: { email: string; password: string }];
  'close': [];
}>();

const show = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value)
});

const form = ref();
const valid = ref(false);
const loading = ref(false);
const email = ref('');
const password = ref('');
const isRegistering = ref(false);
const confirmPassword = ref('');
const loginError = ref('');

const emailRules = [
  (v: string) => !!v || 'Email is required',
  (v: string) => /.+@.+\..+/.test(v) || 'Email must be valid'
];

const passwordRules = [
  (v: string) => !!v || 'Master password is required',
  (v: string) => v.length >= 6 || 'Password must be at least 6 characters'
];

const confirmPasswordRules = [
  (v: string) => !!v || 'Confirm password is required',
  (v: string) => v === password.value || 'Passwords do not match'
];

const handleLogin = async () => {
  if (!valid.value) return;
  
  loading.value = true;
  loginError.value = '';
  
  try {
    if (isRegistering.value) {
      // Register new credentials and generate keys
      console.log('Registering new account for:', email.value);
      await saveLoginCredentials(email.value, password.value);
      
      // Generate PGP keys after saving credentials
      console.log('Generating PGP keys...');
      await init(email.value, password.value);
      console.log('Account setup completed successfully');
      
      emit('login', {
        email: email.value,
        password: password.value
      });
      show.value = false;
    } else {
      // Validate existing credentials
      const isValid = await validateLoginCredentials(email.value, password.value);
      
      if (isValid) {
        emit('login', {
          email: email.value,
          password: password.value
        });
        show.value = false;
      } else {
        loginError.value = 'Invalid email or password';
      }
    }
  } catch (error) {
    console.error('Login/Registration failed:', error);
    loginError.value = 'Setup failed. Please try again.';
  } finally {
    loading.value = false;
  }
};

const handleTempInit = async () => {
  try {
    const dummyEmail = 'test@example.com';
    const dummyPassword = 'temp123456';
    
    await init(dummyEmail, dummyPassword);
    console.log('Temp init completed with dummy data');
  } catch (error) {
    console.error('Temp init failed:', error);
  }
};

const toggleMode = () => {
  isRegistering.value = !isRegistering.value;
  loginError.value = '';
  if (form.value) {
    form.value.reset();
  }
};

onMounted(async () => {
  try {
    const hasCredentials = await hasStoredCredentials();
    if (hasCredentials) {
      const storedEmail = await getStoredEmail();
      if (storedEmail) {
        email.value = storedEmail;
        isRegistering.value = false;
      }
    } else {
      isRegistering.value = true;
    }
  } catch (error) {
    console.error('Failed to check stored credentials:', error);
    isRegistering.value = true;
  }
});

const closeDialog = () => {
  emit('close');
  show.value = false;
};
</script>
<!-- </script> -->
