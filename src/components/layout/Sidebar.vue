<script setup lang="ts">
import { onMounted } from "vue";
import { usePresetsStore } from "../../stores/presets";
import { useUpdaterStore } from "../../stores/updater";

const navItems = [
  { label: "Dashboard", to: "/", icon: "fa-solid fa-gauge-high" },
  { label: "Agents", to: "/agents", icon: "fa-solid fa-users" },
  { label: "Presets", to: "/presets", icon: "fa-solid fa-layer-group" },
  { label: "Settings", to: "/settings", icon: "fa-solid fa-gear" },
];

const presetsStore = usePresetsStore();
const updaterStore = useUpdaterStore();

onMounted(() => {
  presetsStore.fetchAll();
});
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
          <span v-if="item.to === '/settings' && updaterStore.update" class="nav-item-badge" title="Update available"></span>
        </RouterLink>
      </li>
    </ul>

    <div class="separator"></div>

    <div class="preset-section">
      <div class="preset-header">
        <span>Presets</span>
        <RouterLink to="/presets" title="Manage presets">
          <i class="fa-solid fa-plus"></i>
        </RouterLink>
      </div>
      <p v-if="presetsStore.favorites.length === 0" class="preset-empty">No favorite presets yet.</p>
      <RouterLink v-for="preset in presetsStore.favorites" :key="preset.id" to="/presets" class="preset">
        <span>{{ preset.name }}</span>
        <i class="fa-solid fa-star"></i>
      </RouterLink>
    </div>
  </aside>
</template>
