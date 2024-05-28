<script lang="ts" setup>
import { DisplayStateConnection } from "@/api/displayState";
import { useDisplayStateStore } from "@/stores/displayState";
import { computed, onMounted, watch } from "vue";

const props = defineProps({
  displayName: { type: String, required: true },
});

const displayStateStore = useDisplayStateStore();

async function connectDisplayState() {
  await displayStateStore.connect();
}
onMounted(connectDisplayState);
</script>

<template>
  <div>
    {{ displayName }}
    <br />
    <code>
      <pre>{{ displayStateStore.currentState }}</pre>
    </code>
  </div>
</template>

<style lang="scss" scoped></style>
