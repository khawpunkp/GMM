<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { getVersion } from "@tauri-apps/api/app";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { PhGear } from "@phosphor-icons/vue";
import VueButton from "@/components/ui/button/VueButton.vue";
import VueTypography from "@/components/ui/typography/VueTypography.vue";
import { useSettingsStore } from "../../stores/settings";
import { useUpdaterStore } from "../../stores/updater";
import UpdateModal from "../../components/UpdateModal.vue";

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

  unlistenProgress = await listen<{
    processed: number;
    currentPath: string | null;
    message: string;
  }>("scan-progress", (event) => {
    statusMessage.value = event.payload.message;
  });
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
    <div class="mb-6 flex items-center border-b border-white/10 pb-4">
      <VueTypography variant="H1B" as="h1" class="flex items-center gap-3">
        <PhGear :size="20" class="opacity-70" />Settings
      </VueTypography>
    </div>

    <div class="flex flex-col gap-5">
      <div class="w-full rounded-2xl border border-white/10 bg-card p-6">
        <VueTypography variant="TitleB" as="h2" class="mb-2.5"
          >Paths Configuration</VueTypography
        >

        <div
          class="border-b border-white/5 pb-5 mb-5 grid grid-cols-12 items-center"
        >
          <VueTypography variant="BodyB" as="h3" class="col-span-2">
            Mods Folder
          </VueTypography>
          <VueTypography
            variant="BodyR"
            as="p"
            class="break-all text-muted-foreground col-span-8"
          >
            {{ modsFolderPath ?? "Not set" }}
          </VueTypography>
          <div class="flex justify-end col-span-2">
            <VueButton type="button" variant="outlined" @click="chooseFolder">
              Choose Folder
            </VueButton>
          </div>
          <VueTypography
            v-if="statusMessage"
            variant="CaptionR"
            as="p"
            class="mt-3.5 text-muted-foreground"
          >
            {{ statusMessage }}
          </VueTypography>
          <VueTypography
            v-if="errorMessage"
            variant="CaptionR"
            as="p"
            class="mt-3.5 text-destructive"
          >
            {{ errorMessage }}
          </VueTypography>
        </div>

        <div class="grid grid-cols-12 items-center">
          <VueTypography variant="BodyB" as="h3" class="col-span-2">
            Game Executable
          </VueTypography>
          <VueTypography
            variant="BodyR"
            as="p"
            class="break-all text-muted-foreground col-span-8"
          >
            {{ gameExecutablePath ?? "Not set" }}
          </VueTypography>
          <div class="flex justify-end col-span-2">
            <VueButton
              type="button"
              variant="outlined"
              @click="chooseGameExecutable"
              >Choose Executable</VueButton
            >
          </div>
        </div>
      </div>

      <div class="w-full rounded-2xl border border-white/10 bg-card p-6">
        <div class="flex justify-between items-center">
          <div>
            <VueTypography variant="TitleB" as="h2" class="mb-2.5"
              >Mod Management</VueTypography
            >
            <VueTypography variant="BodyR" as="p" class="text-muted-foreground">
              Add new mods and remove deleted mods from the database
            </VueTypography>
          </div>
          <div class="flex items-center justify-start gap-3">
            <VueButton
              type="button"
              :disabled="isScanning || !modsFolderPath"
              @click="runScan"
            >
              {{ isScanning ? "Scanning…" : "Scan Now" }}
            </VueButton>
          </div>
        </div>
      </div>

      <div class="w-full rounded-2xl border border-white/10 bg-card p-6">
        <div class="flex justify-between items-center">
          <div>
            <VueTypography variant="TitleB" as="h2" class="mb-2.5"
              >Updates</VueTypography
            >
            <VueTypography
              variant="BodyR"
              as="p"
              class="mb-4 text-muted-foreground"
            >
              Currently running v{{ currentVersion }}
            </VueTypography>
          </div>
          <div class="flex items-center justify-start gap-3">
            <VueButton
              type="button"
              variant="outlined"
              :disabled="updaterStore.isChecking"
              @click="checkForUpdates"
            >
              {{ updaterStore.isChecking ? "Checking…" : "Check for Updates" }}
            </VueButton>
            <VueButton
              v-if="updaterStore.update"
              type="button"
              @click="showUpdateModal = true"
            >
              Update Available: v{{ updaterStore.update.version }}
            </VueButton>
          </div>
        </div>

        <VueTypography
          v-if="noUpdateFound"
          variant="CaptionR"
          as="p"
          class="text-muted-foreground"
        >
          You're up to date.
        </VueTypography>
        <VueTypography
          v-if="updaterStore.errorMessage"
          variant="CaptionR"
          as="p"
          class="text-destructive"
        >
          {{ updaterStore.errorMessage }}
        </VueTypography>
      </div>
    </div>

    <UpdateModal v-if="showUpdateModal" @close="showUpdateModal = false" />
  </div>
</template>
