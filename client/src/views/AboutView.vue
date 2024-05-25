<script setup lang="ts">
import { API_URI, api } from "@/api/api";
import { useAuthStore } from "@/stores/auth";
import { computed, onMounted, ref } from "vue";

const authStore = useAuthStore();
const isAuthenticated = computed(() => authStore.isAuthenticated);

const license = ref<string>();
async function loadLicense() {
  license.value = await api("server-info/license", "GET", undefined, {
    returnType: "text",
  });
}
onMounted(loadLicense);

const serverVersion = ref<string>();
async function loadVersion() {
  serverVersion.value = await api("server-info/version", "GET", undefined, {
    returnType: "text",
  });
}
onMounted(loadVersion);

const clientVersion = __APP_VERSION__;
</script>

<template>
  <main>
    <h1>StreamSys</h1>

    <h2><em> Garnet DeGelder's Streaming System </em></h2>

    <RouterLink v-if="!isAuthenticated" :to="{ name: 'login' }">
      Login
    </RouterLink>

    <h2>Version:</h2>
    <dl>
      <dt>Client:</dt>
      <dd>
        <code> {{ clientVersion }} </code>
      </dd>

      <dt>Server:</dt>
      <dd>
        <em v-if="serverVersion == undefined"> Loading... </em>
        <code v-else> {{ serverVersion }}</code>
      </dd>
    </dl>

    <h2>License:</h2>
    <code v-if="license != undefined">
      <pre>{{ license }}</pre>
    </code>
    <em v-else> MIT License </em>

    <h2>Source Code:</h2>
    <p>
      The source code for the client and server is hosted on
      <a
        href="https://github.com/g-dg/streamsys"
        target="_blank"
        rel="noopener noreferrer"
      >
        Github
      </a>
    </p>
  </main>
</template>

<style lang="scss" scoped></style>
