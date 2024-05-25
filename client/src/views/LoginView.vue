<script setup lang="ts">
import { AuthClient } from "@/api/auth";
import router from "@/router";
import { useAuthStore } from "@/stores/auth";
import { onMounted, ref } from "vue";

const authStore = useAuthStore();

const username = ref("");
const password = ref("");

const loading = ref(false);
async function login() {
  loading.value = true;
  try {
    const result = await AuthClient.login(username.value, password.value);

    if (result) {
      authStore.token = result.token;
      authStore.user = result.user;

      router.push({ name: "home" });
    } else {
      alert("Login Failed.");
      password.value = "";
    }
  } catch (e) {
    console.error(e);
    alert("Error occurred logging in");
  }
  loading.value = false;
}

onMounted(() => {
  if (authStore.isAuthenticated) {
    router.push({ name: "home" });
  }
});
</script>

<template>
  <main>
    <form @submit.prevent="login" :disabled="loading">
      <h1>StreamSys</h1>
      <h2>Login</h2>

      <input v-model="username" type="text" placeholder="Username" />
      <input v-model="password" type="password" placeholder="Password" />
      <input type="submit" value="Log in" :disabled="loading" />
    </form>

    <br />
    <RouterLink :to="{ name: 'about' }">About</RouterLink>
  </main>
</template>

<style lang="scss" scoped></style>
