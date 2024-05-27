<script lang="ts" setup>
let props = defineProps({
  displayName: { type: String, required: true },
});
</script>

<template>
  {{ displayName }}
</template>

<style lang="scss" scoped></style>
