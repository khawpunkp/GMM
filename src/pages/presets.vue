<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { usePresetsStore } from "../stores/presets";

const presetsStore = usePresetsStore();

const newPresetName = ref("");
const statusMessage = ref<string | null>(null);
const errorMessage = ref<string | null>(null);
const applyingId = ref<number | null>(null);

let unlistenProgress: UnlistenFn | null = null;
let unlistenComplete: UnlistenFn | null = null;
let unlistenError: UnlistenFn | null = null;

onMounted(async () => {
  await presetsStore.fetchAll();

  unlistenProgress = await listen<{ processed: number; total: number; currentModName: string }>(
    "preset-apply-progress",
    (event) => {
      statusMessage.value = `Applying: ${event.payload.currentModName} (${event.payload.processed}/${event.payload.total})`;
    }
  );
  unlistenComplete = await listen<string>("preset-apply-complete", (event) => {
    statusMessage.value = event.payload;
    applyingId.value = null;
  });
  unlistenError = await listen<string>("preset-apply-error", (event) => {
    errorMessage.value = event.payload;
    applyingId.value = null;
  });
});

onUnmounted(() => {
  unlistenProgress?.();
  unlistenComplete?.();
  unlistenError?.();
});

async function createPreset() {
  if (!newPresetName.value.trim()) return;
  errorMessage.value = null;
  try {
    await presetsStore.create(newPresetName.value.trim());
    newPresetName.value = "";
  } catch (e) {
    errorMessage.value = String(e);
  }
}

async function applyPreset(presetId: number) {
  applyingId.value = presetId;
  errorMessage.value = null;
  statusMessage.value = "Applying preset...";
  try {
    await presetsStore.apply(presetId);
  } catch (e) {
    errorMessage.value = String(e);
    applyingId.value = null;
  }
}

async function overwritePreset(presetId: number) {
  if (!confirm("Overwrite this preset with the current mod state?")) return;
  await presetsStore.overwrite(presetId);
}

async function deletePreset(presetId: number, name: string) {
  if (!confirm(`Delete preset "${name}"?`)) return;
  await presetsStore.remove(presetId);
}
</script>

<template>
  <div>
    <div class="page-header">
      <h1 class="page-title"><i class="fa-solid fa-layer-group"></i>Presets</h1>
    </div>

    <div class="card settings-section">
      <h2 class="settings-section-title">Save Current State</h2>
      <p class="settings-value">Snapshots every mod's current enabled/disabled state as a new preset.</p>
      <div class="form-actions settings-actions">
        <input
          v-model="newPresetName"
          class="form-input preset-name-input"
          type="text"
          placeholder="Preset name…"
          @keydown.enter="createPreset"
        />
        <button type="button" class="btn btn-primary" @click="createPreset">Save as Preset</button>
      </div>
      <p v-if="statusMessage" class="settings-status">{{ statusMessage }}</p>
      <p v-if="errorMessage" class="settings-error">{{ errorMessage }}</p>
    </div>

    <div class="preset-list">
      <p v-if="presetsStore.isLoading">Loading…</p>
      <p v-else-if="presetsStore.presets.length === 0" class="settings-value">No presets yet.</p>
      <div v-for="preset in presetsStore.presets" v-else :key="preset.id" class="card preset-row">
        <button
          type="button"
          class="icon-btn"
          :class="{ 'preset-favorite-active': preset.isFavorite }"
          title="Favorite"
          @click="presetsStore.toggleFavorite(preset.id, !preset.isFavorite)"
        >
          <i class="fa-solid fa-star"></i>
        </button>
        <span class="preset-name">{{ preset.name }}</span>
        <div class="preset-actions">
          <button
            type="button"
            class="btn btn-primary"
            :disabled="applyingId === preset.id"
            @click="applyPreset(preset.id)"
          >
            {{ applyingId === preset.id ? "Applying…" : "Apply" }}
          </button>
          <button type="button" class="btn btn-secondary" @click="overwritePreset(preset.id)">Overwrite</button>
          <button type="button" class="btn btn-danger" @click="deletePreset(preset.id, preset.name)">Delete</button>
        </div>
      </div>
    </div>
  </div>
</template>
