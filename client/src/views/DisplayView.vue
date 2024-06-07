<script lang="ts" setup>
import { useStateStore } from "@/stores/state";
import { onMounted, ref } from "vue";

const props = defineProps({
  displayName: { type: String, required: true },
});

const stateStore = useStateStore();

async function connectState() {
  await stateStore.connect();
}
onMounted(connectState);

const newStateID = ref("");
async function setState() {
  await stateStore.authenticate();
  stateStore.setState({
    id: newStateID.value,
    display: {
      content: {},
      slide_type_id: null,
    },
  });
}
</script>

<template>
  <div>
    {{ displayName }}
    <br />
    <form @submit.prevent="setState">
      <label>ID:</label>
      <input v-model="newStateID" type="text" />
      <input type="submit" value="Set" />
    </form>
    <br />
    <code>
      <pre>{{ stateStore.currentState }}</pre>
    </code>
  </div>
</template>

<style lang="scss" scoped></style>
