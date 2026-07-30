<script setup lang="ts">
import Sidebar from "../components/layout/Sidebar.vue";
</script>

<template>
  <div class="flex h-screen bg-black/20">
    <Sidebar />
    <main class="relative flex-1 overflow-y-auto p-6">
      <RouterView />
    </main>
  </div>
</template>
