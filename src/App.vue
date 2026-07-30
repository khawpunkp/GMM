<script setup lang="ts">
import { onMounted } from "vue";
import AppShell from "./layouts/AppShell.vue";
import { useUpdaterStore } from "./stores/updater";

const updaterStore = useUpdaterStore();

onMounted(() => {
  // Silent startup check — just populates updaterStore.update for the Sidebar badge.
  // No modal pops up unprompted; the user opens it via the badge/Settings.
  // check() never rejects (errors are caught internally onto updaterStore.errorMessage).
  updaterStore.check();
});
</script>

<template>
  <AppShell />
</template>
