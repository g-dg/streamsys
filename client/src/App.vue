<script setup lang="ts">
import { RouterLink, RouterView } from "vue-router";
import { useAuthStore } from "./stores/auth";
import { computed } from "vue";
import { UserPermission } from "./api/users";
import router from "./router";

const authStore = useAuthStore();

const isAuthenticated = computed(() => authStore.isAuthenticated);
const user = computed(() => authStore.user);

const showHeaderFooter = computed(
  () =>
    router.currentRoute.value.meta.showHeaderFooter ??
    router.currentRoute.value.matched.length > 0
);

const clientVersion = __APP_VERSION__;
</script>

<template>
  <div>
    <header v-if="showHeaderFooter">
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

    <footer v-if="showHeaderFooter">
      StreamSys Copyright &copy; 2024 Garnet DeGelder
    </footer>
  </div>
</template>

<style lang="scss" scoped>
footer {
  margin-top: 2em;
}
</style>
