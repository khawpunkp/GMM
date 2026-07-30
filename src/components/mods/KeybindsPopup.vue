<script setup lang="ts">
import { onMounted, reactive, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { KeybindInfo, PersistVar } from "../../types";

const props = defineProps<{ modId: number }>();
const emit = defineEmits<{ close: [] }>();

const keybinds = ref<KeybindInfo[]>([]);
const persistVars = ref<PersistVar[]>([]);
const isLoading = ref(true);
const errorMessage = ref<string | null>(null);

const savingVars = reactive<Record<string, boolean>>({});
const varErrors = reactive<Record<string, string>>({});

onMounted(async () => {
  try {
    const [kb, pv] = await Promise.all([
      invoke<KeybindInfo[]>("get_mod_keybinds", { modId: props.modId }),
      invoke<PersistVar[]>("get_mod_persist_vars", { modId: props.modId }),
    ]);
    keybinds.value = kb;
    persistVars.value = pv;
  } catch (e) {
    errorMessage.value = String(e);
  } finally {
    isLoading.value = false;
  }
});

async function saveVar(varName: string, rawValue: string) {
  const value = Number(rawValue);
  if (!Number.isInteger(value)) return;

  savingVars[varName] = true;
  varErrors[varName] = "";
  try {
    await invoke("set_mod_persist_var", { modId: props.modId, varName, value });
    const target = persistVars.value.find((v) => v.name === varName);
    if (target) target.value = value;
  } catch (e) {
    varErrors[varName] = String(e);
  } finally {
    savingVars[varName] = false;
  }
}
</script>

<template>
  <div class="modal-overlay" @click.self="emit('close')">
    <div class="modal-content card">
      <h2 class="settings-section-title">Keybinds</h2>

      <p v-if="isLoading">Loading…</p>
      <template v-else-if="errorMessage">
        <p class="settings-error">{{ errorMessage }}</p>
      </template>
      <template v-else>
        <p v-if="keybinds.length === 0" class="settings-value">
          No keybinds found — this mod's INI has no "; Constants" section, or none of its [Key...]
          sections have a value set yet.
        </p>
        <ul v-else class="keybind-list">
          <li v-for="kb in keybinds" :key="kb.title" class="keybind-row">
            <span class="keybind-title">{{ kb.title }}</span>
            <span class="keybind-key">{{ kb.key }}</span>
          </li>
        </ul>

        <template v-if="persistVars.length > 0">
          <h2 class="settings-section-title">Toggle memory</h2>
          <div v-for="pv in persistVars" :key="pv.name" class="form-group">
            <label class="form-label" :for="`persist-${pv.name}`">{{ pv.name }}</label>
            <select
              v-if="pv.options.length > 0"
              :id="`persist-${pv.name}`"
              class="form-input"
              :value="pv.value"
              :disabled="savingVars[pv.name]"
              @change="saveVar(pv.name, ($event.target as HTMLSelectElement).value)"
            >
              <option v-for="opt in pv.options" :key="opt" :value="opt">{{ opt }}</option>
            </select>
            <input
              v-else
              :id="`persist-${pv.name}`"
              class="form-input"
              type="number"
              :value="pv.value"
              :disabled="savingVars[pv.name]"
              @change="saveVar(pv.name, ($event.target as HTMLInputElement).value)"
            />
            <p v-if="varErrors[pv.name]" class="settings-error">{{ varErrors[pv.name] }}</p>
          </div>
        </template>
      </template>

      <div class="form-actions">
        <button type="button" class="btn btn-secondary" @click="emit('close')">Close</button>
      </div>
    </div>
  </div>
</template>
