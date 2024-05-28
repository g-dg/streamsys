<script lang="ts" setup>
import { DisplayStateConnection } from "@/api/displayState";
import { onMounted, watch } from "vue";

const props = defineProps({
  displayName: { type: String, required: true },
});

let displayStateConnection: DisplayStateConnection | null;
async function connectDisplayState() {
  displayStateConnection = await DisplayStateConnection.connect();
}
onMounted(connectDisplayState);
</script>

<template>
  <div>
  {{ displayName }}
  <br />
  <code>
    <pre>{{ displayStateConnection?.currentState.value }}</pre>
  </code>
  </div>
</template>

<style lang="scss" scoped></style>
