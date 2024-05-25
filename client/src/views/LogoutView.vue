<script setup lang="ts">
import { AuthClient } from "@/api/auth";
import router from "@/router";
import { useAuthStore } from "@/stores/auth";
import { onMounted } from "vue";

const authStore = useAuthStore();

onMounted(async () => {
  if (authStore.isAuthenticated) {
    try {
      await AuthClient.logout();
    } catch {}
    authStore.user = null;
    authStore.token = null;
  }

  router.push({ name: "login" });
});
</script>

<template>
  <main></main>
</template>

<style lang="scss" scoped></style>
