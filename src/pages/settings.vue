<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { getVersion } from "@tauri-apps/api/app";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useSettingsStore } from "../stores/settings";
import { useUpdaterStore } from "../stores/updater";
import UpdateModal from "../components/UpdateModal.vue";

const settingsStore = useSettingsStore();
const updaterStore = useUpdaterStore();

const modsFolderPath = ref<string | null>(null);
const isScanning = ref(false);
const statusMessage = ref<string | null>(null);
const errorMessage = ref<string | null>(null);

const gameExecutablePath = ref<string | null>(null);

const currentVersion = ref<string>("");
const showUpdateModal = ref(false);
const noUpdateFound = ref(false);

let unlistenProgress: UnlistenFn | null = null;
let unlistenComplete: UnlistenFn | null = null;
let unlistenError: UnlistenFn | null = null;

onMounted(async () => {
  modsFolderPath.value = await settingsStore.fetch("mods_folder_path");
  gameExecutablePath.value = await settingsStore.fetch("game_executable_path");
  currentVersion.value = await getVersion();

  unlistenProgress = await listen<{ processed: number; currentPath: string | null; message: string }>(
    "scan-progress",
    (event) => {
      statusMessage.value = event.payload.message;
    }
  );
  unlistenComplete = await listen<string>("scan-complete", (event) => {
    statusMessage.value = event.payload;
    isScanning.value = false;
  });
  unlistenError = await listen<string>("scan-error", (event) => {
    errorMessage.value = event.payload;
    isScanning.value = false;
  });
});

onUnmounted(() => {
  unlistenProgress?.();
  unlistenComplete?.();
  unlistenError?.();
});

async function chooseFolder() {
  const path = await open({ directory: true, multiple: false });
  if (typeof path === "string") {
    await settingsStore.set("mods_folder_path", path);
    modsFolderPath.value = path;
  }
}

async function runScan() {
  if (!modsFolderPath.value) {
    errorMessage.value = "Set a mods folder first.";
    return;
  }
  isScanning.value = true;
  errorMessage.value = null;
  statusMessage.value = "Starting scan...";
  try {
    const summary = await invoke<string>("scan_mods_directory");
    statusMessage.value = summary;
  } catch (e) {
    errorMessage.value = String(e);
  } finally {
    isScanning.value = false;
  }
}

async function chooseGameExecutable() {
  const path = await open({
    multiple: false,
    filters: [{ name: "Executable", extensions: ["exe"] }],
  });
  if (typeof path === "string") {
    await settingsStore.set("game_executable_path", path);
    gameExecutablePath.value = path;
  }
}

async function checkForUpdates() {
  noUpdateFound.value = false;
  const update = await updaterStore.check();
  if (update) {
    showUpdateModal.value = true;
  } else if (!updaterStore.errorMessage) {
    noUpdateFound.value = true;
  }
}
</script>

<template>
  <div>
    <div class="page-header">
      <h1 class="page-title"><i class="fa-solid fa-gear"></i>Settings</h1>
    </div>

    <div class="card settings-section">
      <h2 class="settings-section-title">Mods Folder</h2>
      <p class="settings-value">{{ modsFolderPath ?? "Not set" }}</p>
      <div class="form-actions settings-actions">
        <button type="button" class="btn btn-secondary" @click="chooseFolder">Choose Folder</button>
        <button type="button" class="btn btn-primary" :disabled="isScanning || !modsFolderPath" @click="runScan">
          {{ isScanning ? "Scanning…" : "Scan Now" }}
        </button>
      </div>
      <p v-if="statusMessage" class="settings-status">{{ statusMessage }}</p>
      <p v-if="errorMessage" class="settings-error">{{ errorMessage }}</p>
    </div>

    <div class="card settings-section">
      <h2 class="settings-section-title">Game Executable</h2>
      <p class="settings-value">{{ gameExecutablePath ?? "Not set" }}</p>
      <div class="form-actions settings-actions">
        <button type="button" class="btn btn-secondary" @click="chooseGameExecutable">Choose Executable</button>
      </div>
    </div>

    <div class="card settings-section">
      <h2 class="settings-section-title">Updates</h2>
      <p class="settings-value">Currently running v{{ currentVersion }}</p>
      <div class="form-actions settings-actions">
        <button type="button" class="btn btn-secondary" :disabled="updaterStore.isChecking" @click="checkForUpdates">
          {{ updaterStore.isChecking ? "Checking…" : "Check for Updates" }}
        </button>
        <button v-if="updaterStore.update" type="button" class="btn btn-primary" @click="showUpdateModal = true">
          Update Available: v{{ updaterStore.update.version }}
        </button>
      </div>
      <p v-if="noUpdateFound" class="settings-status">You're up to date.</p>
      <p v-if="updaterStore.errorMessage" class="settings-error">{{ updaterStore.errorMessage }}</p>
    </div>

    <UpdateModal v-if="showUpdateModal" @close="showUpdateModal = false" />
  </div>
</template>
