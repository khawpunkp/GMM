<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import { invoke } from "@tauri-apps/api/core";
import { PhUsers, PhGear, PhPlay, PhFileArrowDown, PhFolderOpen, PhDotsThreeOutline } from "@phosphor-icons/vue";
import VueButton from "@/components/ui/button/VueButton.vue";
import VueTypography from "@/components/ui/typography/VueTypography.vue";
import { useSettingsStore } from "../../stores/settings";
import { useUpdaterStore } from "../../stores/updater";
import { CATEGORY_ICONS } from "../../utils/category";
import ImportModal from "../mods/ImportModal.vue";

const navItems = [
  { label: "Agents", to: "/agents", icon: PhUsers },
  { label: "NPCs", to: "/categories/npcs", icon: CATEGORY_ICONS.npcs },
  { label: "Enemies", to: "/categories/enemies", icon: CATEGORY_ICONS.enemies },
  { label: "Weapons", to: "/categories/weapons", icon: CATEGORY_ICONS.weapons },
  { label: "Objects", to: "/categories/objects", icon: CATEGORY_ICONS.objects },
  { label: "UI", to: "/categories/ui", icon: CATEGORY_ICONS.ui },
  { label: "Other", to: "/other", icon: PhDotsThreeOutline },
  { label: "Settings", to: "/settings", icon: PhGear },
];

const route = useRoute();
const router = useRouter();
const settingsStore = useSettingsStore();
const updaterStore = useUpdaterStore();

function isActive(path: string) {
  return route.path === path || route.path.startsWith(path + "/");
}

const gameExecutablePath = ref<string | null>(null);
const isLaunching = ref(false);
const launchError = ref<string | null>(null);
const isImporting = ref(false);
const openFolderError = ref<string | null>(null);

onMounted(async () => {
  gameExecutablePath.value = await settingsStore.fetch("game_executable_path");
});

async function launchGame() {
  isLaunching.value = true;
  launchError.value = null;
  try {
    await invoke("launch_game");
  } catch (e) {
    launchError.value = String(e);
  } finally {
    isLaunching.value = false;
  }
}

async function openModsFolder() {
  openFolderError.value = null;
  try {
    await invoke("open_mods_folder");
  } catch (e) {
    openFolderError.value = String(e);
  }
}
</script>

<template>
  <aside
    class="flex h-full w-65 shrink-0 flex-col overflow-y-auto border-r border-white/10 bg-card p-5"
  >
    <div class="mb-8 flex items-center justify-center">
      <VueTypography
        variant="H1B"
        as="span"
        class="bg-linear-to-br from-primary to-secondary bg-clip-text text-transparent"
      >
        GMM
      </VueTypography>
    </div>

    <VueButton
      class="w-full justify-center"
      :disabled="isLaunching || !gameExecutablePath"
      @click="launchGame"
    >
      <PhPlay :size="24" weight="fill" />
      {{ isLaunching ? "Launching…" : "Quick Launch" }}
    </VueButton>
    <VueTypography
      v-if="launchError"
      variant="CaptionR"
      as="p"
      class="mt-2 mb-2.5 text-destructive"
    >
      {{ launchError }}
    </VueTypography>

    <VueButton
      variant="outlined"
      class="mt-2.5 w-full justify-center"
      @click="isImporting = true"
    >
      <PhFileArrowDown :size="24" weight="fill" />
      Import Mod
    </VueButton>

    <ul class="mt-6 grow list-none">
      <li v-for="item in navItems" :key="item.to" class="mb-2">
        <VueButton
          type="button"
          variant="ghost"
          color="gray"
          class="w-full justify-start gap-3 rounded-lg px-3.5 py-3"
          :class="
            isActive(item.to)
              ? 'bg-primary text-white shadow-[0_5px_15px_rgba(156,136,255,0.4)]'
              : 'hover:bg-primary/10'
          "
          @click="router.push(item.to)"
        >
          <component :is="item.icon" :size="24" weight="fill" />
          {{ item.label }}
          <span
            v-if="item.to === '/settings' && updaterStore.update"
            class="ml-auto size-2 rounded-full bg-accent"
            title="Update available"
          />
        </VueButton>
      </li>
    </ul>

    <VueButton
      variant="outlined"
      class="mt-4 w-full justify-center"
      @click="openModsFolder"
    >
      <PhFolderOpen :size="24" weight="fill" />
      Open Mods Folder
    </VueButton>
    <VueTypography
      v-if="openFolderError"
      variant="CaptionR"
      as="p"
      class="mt-2 text-destructive"
    >
      {{ openFolderError }}
    </VueTypography>

    <ImportModal
      v-if="isImporting"
      @imported="isImporting = false"
      @close="isImporting = false"
    />
  </aside>
</template>
