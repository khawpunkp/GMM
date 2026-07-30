<script setup lang="ts">
import { onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useSettingsStore } from "../../stores/settings";
import { useModsStore } from "../../stores/mods";
import type { Mod } from "../../types";

const props = defineProps<{
  mod: Mod;
  selectMode?: boolean;
  selected?: boolean;
}>();
const emit = defineEmits<{
  edit: [mod: Mod];
  delete: [mod: Mod];
  keybinds: [mod: Mod];
  "toggle-select": [mod: Mod];
}>();

const settingsStore = useSettingsStore();
const modsStore = useModsStore();
const imageSrc = ref<string | null>(null);

onMounted(async () => {
  if (!props.mod.imageFilename) return;
  const modsFolderPath = settingsStore.settings.mods_folder_path ?? (await settingsStore.fetch("mods_folder_path"));
  if (!modsFolderPath) return;
  const fullPath = `${modsFolderPath}/${props.mod.folderName}/${props.mod.imageFilename}`;
  try {
    imageSrc.value = await invoke<string>("read_image_as_data_url", { path: fullPath });
  } catch {
    imageSrc.value = null;
  }
});

function toggle() {
  modsStore.toggle(props.mod.id);
}

function openFolder() {
  modsStore.openFolder(props.mod.id);
}
</script>

<template>
  <div
    class="card mod-card"
    :class="{ 'mod-card-disabled': !mod.isEnabled, 'mod-card-selected': selectMode && selected }"
    @click="selectMode && emit('toggle-select', mod)"
  >
    <input v-if="selectMode" type="checkbox" class="mod-card-select-checkbox" :checked="selected" readonly />
    <img :src="imageSrc ?? '/images/placeholder.jpg'" alt="" class="mod-card-image" />
    <div class="mod-card-body">
      <div class="mod-card-name">{{ mod.name }}</div>
      <div v-if="mod.author" class="mod-card-author">by {{ mod.author }}</div>
    </div>
    <div v-if="!selectMode" class="mod-card-actions">
      <label class="switch" :title="mod.isEnabled ? 'Enabled' : 'Disabled'">
        <input type="checkbox" :checked="mod.isEnabled" @change="toggle" />
        <span class="switch-slider"></span>
      </label>
      <button type="button" class="icon-btn" title="Edit" @click="emit('edit', mod)">
        <i class="fa-solid fa-pen"></i>
      </button>
      <button type="button" class="icon-btn" title="Open folder" @click="openFolder">
        <i class="fa-solid fa-folder-open"></i>
      </button>
      <button type="button" class="icon-btn" title="Keybinds" @click="emit('keybinds', mod)">
        <i class="fa-solid fa-keyboard"></i>
      </button>
      <button type="button" class="icon-btn icon-btn-danger" title="Delete" @click="emit('delete', mod)">
        <i class="fa-solid fa-trash"></i>
      </button>
    </div>
  </div>
</template>
