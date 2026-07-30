<script setup lang="ts">
import { computed } from "vue";
import { useUpdaterStore } from "../stores/updater";

const emit = defineEmits<{ close: [] }>();
const updaterStore = useUpdaterStore();

const progressPercent = computed(() => {
  if (!updaterStore.totalBytes) return null;
  return Math.min(100, Math.round((updaterStore.downloadedBytes / updaterStore.totalBytes) * 100));
});

async function handleClose() {
  if (updaterStore.isDownloading) return;
  await updaterStore.dismiss();
  emit("close");
}

async function handleInstall() {
  await updaterStore.downloadAndInstall();
}

async function handleRestart() {
  await updaterStore.restart();
}
</script>

<template>
  <div class="modal-overlay" @click.self="handleClose">
    <div class="modal-content card">
      <h2 class="settings-section-title">Update Available: v{{ updaterStore.update?.version }}</h2>
      <p class="settings-value">Currently running v{{ updaterStore.update?.currentVersion }}</p>
      <p v-if="updaterStore.update?.body" class="update-notes">{{ updaterStore.update.body }}</p>

      <div v-if="updaterStore.isDownloading" class="update-progress">
        <div class="update-progress-bar">
          <div class="update-progress-fill" :style="{ width: (progressPercent ?? 0) + '%' }"></div>
        </div>
        <p class="settings-status">{{ progressPercent !== null ? `${progressPercent}%` : "Downloading…" }}</p>
      </div>

      <p v-if="updaterStore.errorMessage" class="settings-error">{{ updaterStore.errorMessage }}</p>

      <div class="form-actions">
        <button v-if="updaterStore.isReadyToRestart" type="button" class="btn btn-primary" @click="handleRestart">
          Restart Now
        </button>
        <template v-else>
          <button type="button" class="btn btn-secondary" :disabled="updaterStore.isDownloading" @click="handleClose">
            Later
          </button>
          <button type="button" class="btn btn-primary" :disabled="updaterStore.isDownloading" @click="handleInstall">
            {{ updaterStore.isDownloading ? "Downloading…" : "Download & Install" }}
          </button>
        </template>
      </div>
    </div>
  </div>
</template>
