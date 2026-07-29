<script setup lang="ts">
const navItems = [
  { label: "Dashboard", to: "/", icon: "fa-solid fa-gauge-high" },
  { label: "Agents", to: "/agents", icon: "fa-solid fa-users" },
  { label: "Presets", to: "/presets", icon: "fa-solid fa-layer-group" },
  { label: "Settings", to: "/settings", icon: "fa-solid fa-gear" },
];
</script>

<template>
  <aside class="sidebar">
    <div class="logo">
      <span>GMM</span>
    </div>

    <ul class="nav-items">
      <li v-for="item in navItems" :key="item.to">
        <RouterLink :to="item.to" class="nav-item" active-class="active">
          <i :class="item.icon"></i>
          {{ item.label }}
        </RouterLink>
      </li>
    </ul>

    <div class="separator"></div>

    <div class="preset-section">
      <div class="preset-header">
        <span>Presets</span>
        <button type="button" title="New preset">
          <i class="fa-solid fa-plus"></i>
        </button>
      </div>
      <p class="preset-empty">No presets yet.</p>
    </div>
  </aside>
</template>
