<script lang="ts" setup>
import { useDisplayStateStore } from "@/stores/displayState";
import { onMounted, ref } from "vue";

const props = defineProps({
  displayName: { type: String, required: true },
});

const displayStateStore = useDisplayStateStore();

async function connectDisplayState() {
  await displayStateStore.connect();
}
onMounted(connectDisplayState);

const newStateID = ref("");
async function setState() {
  await displayStateStore.authenticate();
  displayStateStore.setState({
    id: newStateID.value,
    content: {},
    slide_type_id: null,
  });
}
</script>

<template>
  <div>
    {{ displayName }}
    <br />
    <label>ID:</label>
    <input v-model="newStateID" type="text" />
    <input @click="setState" type="submit" value="Set" />
    <br />
    <code>
      <pre>{{ displayStateStore.currentState }}</pre>
    </code>
  </div>
</template>

<style lang="scss" scoped></style>
