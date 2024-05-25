import "./assets/main.css";

import { createApp } from "vue";
import { createPinia } from "pinia";

import App from "./App.vue";
import router from "./router";

import { useAuthStore } from "./stores/auth";

(async () => {
  const app = createApp(App);

  app.use(createPinia());
  app.use(router);

  const authStore = useAuthStore();

  // if user is not authenticated, redirect to login
  if (!authStore.isAuthenticated) {
    await router.push({ name: "login" });
  }

  app.mount("#app");
})();
