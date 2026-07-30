<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useSettingsStore } from "../stores/settings";
import type { Mod } from "../types";

const settingsStore = useSettingsStore();

const gameExecutablePath = ref<string | null>(null);
const allMods = ref<Mod[]>([]);
const isLaunching = ref(false);
const isLoading = ref(true);
const errorMessage = ref<string | null>(null);

const enabledCount = computed(() => allMods.value.filter((m) => m.isEnabled).length);
const uncategorizedCount = computed(() => allMods.value.filter((m) => !m.agentId && !m.categoryId).length);

onMounted(async () => {
  try {
    gameExecutablePath.value = await settingsStore.fetch("game_executable_path");
    allMods.value = await invoke<Mod[]>("list_mods", { agentId: null, categoryId: null, categoryItemId: null });
  } finally {
    isLoading.value = false;
  }
});

async function launchGame() {
  isLaunching.value = true;
  errorMessage.value = null;
  try {
    await invoke("launch_game");
  } catch (e) {
    errorMessage.value = String(e);
  } finally {
    isLaunching.value = false;
  }
}
</script>

<template>
  <div>
    <div class="page-header">
      <h1 class="page-title"><i class="fa-solid fa-gauge-high"></i>Dashboard</h1>
    </div>

    <div class="card settings-section dashboard-launch">
      <template v-if="gameExecutablePath">
        <button
          type="button"
          class="btn btn-primary dashboard-launch-btn"
          :disabled="isLaunching"
          @click="launchGame"
        >
          <i class="fa-solid fa-play"></i>
          {{ isLaunching ? "Launching…" : "Launch Game" }}
        </button>
      </template>
      <p v-else class="settings-value">
        No game executable configured yet. <RouterLink to="/settings">Set one in Settings</RouterLink>
        to launch from here.
      </p>
      <p v-if="errorMessage" class="settings-error">{{ errorMessage }}</p>
    </div>

    <div v-if="!isLoading" class="dashboard-stats">
      <div class="card dashboard-stat">
        <div class="dashboard-stat-value">{{ allMods.length }}</div>
        <div class="dashboard-stat-label">Total mods</div>
      </div>
      <div class="card dashboard-stat">
        <div class="dashboard-stat-value">{{ enabledCount }}</div>
        <div class="dashboard-stat-label">Enabled</div>
      </div>
      <div class="card dashboard-stat">
        <div class="dashboard-stat-value">{{ uncategorizedCount }}</div>
        <div class="dashboard-stat-label">Uncategorized</div>
      </div>
    </div>
  </div>
</template>
