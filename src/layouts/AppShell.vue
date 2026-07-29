<script setup lang="ts">
import Sidebar from "../components/layout/Sidebar.vue";
</script>

<template>
  <div class="app-container">
    <Sidebar />
    <main class="main-content">
      <RouterView />
    </main>
  </div>
</template>
