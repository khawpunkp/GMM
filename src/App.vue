<script setup lang="ts">
import { onMounted, ref } from 'vue';
import AppShell from './layouts/AppShell.vue';
import SetupRequiredModal from './components/SetupRequiredModal.vue';
import { useSettingsStore } from './stores/settings';
import { useUpdaterStore } from './stores/updater';

const updaterStore = useUpdaterStore();
const settingsStore = useSettingsStore();

const missingModsFolder = ref(false);
const missingGameExecutable = ref(false);
// Shown once per launch. Its only action navigates to Settings, so it won't re-prompt mid-session
// if the user leaves that page with the paths still unset.
const showSetupRequired = ref(false);

onMounted(async () => {
   // Silent startup check — just populates updaterStore.update for the Sidebar badge.
   // No modal pops up unprompted; the user opens it via the badge/Settings.
   // check() never rejects (errors are caught internally onto updaterStore.errorMessage).
   updaterStore.check();

   const [modsFolder, gameExecutable] = await Promise.all([
      settingsStore.fetch('mods_folder_path'),
      settingsStore.fetch('game_executable_path'),
   ]);
   missingModsFolder.value = !modsFolder;
   missingGameExecutable.value = !gameExecutable;
   showSetupRequired.value = missingModsFolder.value || missingGameExecutable.value;
});
</script>

<template>
   <AppShell />

   <SetupRequiredModal
      v-if="showSetupRequired"
      :missing-mods-folder="missingModsFolder"
      :missing-game-executable="missingGameExecutable"
      @close="showSetupRequired = false"
   />
</template>
