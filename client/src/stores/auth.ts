import { ref, computed, watch } from "vue";
import { defineStore } from "pinia";
import type { User } from "@/api/users";
import router from "@/router";

interface AuthStorage {
  token: string | null;
  user: User | null;
}

export const useAuthStore = defineStore("auth", () => {
  const AUTH_LOCALSTORAGE_KEY = "StreamSys_Auth";

  const token = ref<string | null>(null);

  const user = ref<User | null>(null);

  const isAuthenticated = computed(() => token.value != null);

  function loadFromLocalStorage() {
    if (token.value == null) {
      const storage = getStorage();
      token.value = storage.token;
      user.value = storage.user;
    }
  }

  loadFromLocalStorage();

  watch(token, () => {
    if (token.value != null) {
      setStorage({ token: token.value, user: user.value });
    } else {
      removeStorage();
    }
  });

  watch(user, () => {
    setStorage({ token: token.value, user: user.value });
  });

  window.addEventListener("storage", (evt) => {
    if (evt.key === AUTH_LOCALSTORAGE_KEY && evt.oldValue != evt.newValue) {
      const storage = getStorage();
      token.value = storage.token;
      if (token.value == null) {
        user.value = null;
        router.push({ name: "logout" });
      } else {
        user.value = storage.user;
      }
    }
  });

  function getStorage(): AuthStorage {
    const json =
      window.localStorage.getItem(AUTH_LOCALSTORAGE_KEY) ??
      JSON.stringify({ token: null, user: null });
    return JSON.parse(json);
  }

  function setStorage(authObj: AuthStorage) {
    window.localStorage.setItem(AUTH_LOCALSTORAGE_KEY, JSON.stringify(authObj));
  }

  function removeStorage() {
    window.localStorage.removeItem(AUTH_LOCALSTORAGE_KEY);
  }

  return {
    token,
    user,
    isAuthenticated,
  };
});
