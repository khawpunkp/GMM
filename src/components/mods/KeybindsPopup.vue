<script setup lang="ts">
import { onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { KeybindInfo } from "../../types";

const props = defineProps<{ modId: number }>();
const emit = defineEmits<{ close: [] }>();

const keybinds = ref<KeybindInfo[]>([]);
const isLoading = ref(true);
const errorMessage = ref<string | null>(null);

onMounted(async () => {
  try {
    keybinds.value = await invoke<KeybindInfo[]>("get_mod_keybinds", { modId: props.modId });
  } catch (e) {
    errorMessage.value = String(e);
  } finally {
    isLoading.value = false;
  }
});
</script>

<template>
  <div class="modal-overlay" @click.self="emit('close')">
    <div class="modal-content card">
      <h2 class="settings-section-title">Keybinds</h2>

      <p v-if="isLoading">Loading…</p>
      <p v-else-if="errorMessage" class="settings-error">{{ errorMessage }}</p>
      <p v-else-if="keybinds.length === 0" class="settings-value">
        No keybinds found — this mod's INI has no "; Constants" section, or none of its [Key...]
        sections have a value set yet.
      </p>
      <ul v-else class="keybind-list">
        <li v-for="kb in keybinds" :key="kb.title" class="keybind-row">
          <span class="keybind-title">{{ kb.title }}</span>
          <span class="keybind-key">{{ kb.key }}</span>
        </li>
      </ul>

      <div class="form-actions">
        <button type="button" class="btn btn-secondary" @click="emit('close')">Close</button>
      </div>
    </div>
  </div>
</template>
