<script setup lang="ts">
import { useRoute } from "vue-router";

const route = useRoute();
</script>

<template>
  <div>
    <div class="page-header">
      <h1 class="page-title"><i class="fa-solid fa-user"></i>Agent: {{ route.params.slug }}</h1>
    </div>
    <p>Agent detail view coming in a later phase.</p>
  </div>
</template>
