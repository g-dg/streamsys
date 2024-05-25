<script setup lang="ts">
import { RouterLink, RouterView } from "vue-router";
import { useAuthStore } from "./stores/auth";
import { computed } from "vue";
import { UserPermission } from "./api/users";

const authStore = useAuthStore();

const isAuthenticated = computed(() => authStore.isAuthenticated);
const user = computed(() => authStore.user);

const clientVersion = __APP_VERSION__;
</script>

<template>
  <div>
    <header>
      <nav v-if="isAuthenticated">
        <ul>
          <li>
            <RouterLink :to="{ name: 'home' }"> Home </RouterLink>
          </li>
          <li v-if="(user?.permissions ?? 0) & UserPermission.USER_ADMIN">
            <RouterLink :to="{ name: 'user_list' }"> Users </RouterLink>
          </li>
          <li v-if="(user?.permissions ?? 0) & UserPermission.MODIFY_SELF">
            <RouterLink :to="{ name: 'account' }"> Account </RouterLink>
          </li>
          <li>
            <RouterLink :to="{ name: 'about' }"> About </RouterLink>
          </li>
          <li>
            <RouterLink :to="{ name: 'logout' }"> Logout </RouterLink>
          </li>
        </ul>
      </nav>
    </header>

    <main>
      <RouterView />
    </main>

    <footer>StreamSys Copyright &copy; 2024 Garnet DeGelder</footer>
  </div>
</template>

<style lang="scss" scoped>
footer {
  margin-top: 2em;
}
</style>
