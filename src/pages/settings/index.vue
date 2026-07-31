<script setup lang="ts">
import { onMounted, ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { getVersion } from "@tauri-apps/api/app";
import { PhGear } from "@phosphor-icons/vue";
import VueButton from "@/components/ui/button/VueButton.vue";
import VueTypography from "@/components/ui/typography/VueTypography.vue";
import { useSettingsStore } from "../../stores/settings";
import { useUpdaterStore } from "../../stores/updater";
import UpdateModal from "../../components/UpdateModal.vue";

const settingsStore = useSettingsStore();
const updaterStore = useUpdaterStore();

const modsFolderPath = ref<string | null>(null);
const gameExecutablePath = ref<string | null>(null);

const currentVersion = ref<string>("");
const showUpdateModal = ref(false);
const noUpdateFound = ref(false);

onMounted(async () => {
  modsFolderPath.value = await settingsStore.fetch("mods_folder_path");
  gameExecutablePath.value = await settingsStore.fetch("game_executable_path");
  currentVersion.value = await getVersion();
});

async function chooseFolder() {
  const path = await open({ directory: true, multiple: false });
  if (typeof path === "string") {
    await settingsStore.set("mods_folder_path", path);
    modsFolderPath.value = path;
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
    <div class="mb-6 flex items-center border-b border-white/10 pb-4 ">
      <VueTypography variant="H1B" as="h1" class="flex items-center gap-3 h-12">
        <PhGear :size="32" weight="fill"/>Settings
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
