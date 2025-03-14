<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

const firstName = ref("");
const lastName = ref("");
const email = ref("");
const password = ref("");

async function joinNow() {
  // Implement the join now functionality
  console.log("Join now clicked");
}

async function continueWithFacebook() {
  // Implement the continue with Facebook functionality
  console.log("Continue with Facebook clicked");
}
</script>

<template>
  <main class="container">
    <h1>Make the most of your professional life</h1>

    <form class="form" @submit.prevent="joinNow">
      <input id="first-name" v-model="firstName" placeholder="First name" />
      <input id="last-name" v-model="lastName" placeholder="Last name" />
      <input id="email" v-model="email" placeholder="Email" />
      <input id="password" v-model="password" type="password" placeholder="Password (6 or more characters)" />
      <p>By clicking Join now, you agree to LinkedIn's <a href="#">User Agreement</a>, <a href="#">Privacy Policy</a>, and <a href="#">Cookie Policy</a>.</p>
      <button type="submit">Join now</button>
    </form>

    <div class="divider">or</div>

    <button class="facebook-button" @click="continueWithFacebook">
      <img src="./assets/facebook.svg" alt="Facebook logo" />
      Continue with Facebook
    </button>

    <p>Already on LinkedIn? <a href="#">Sign in</a></p>
  </main>
</template>

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