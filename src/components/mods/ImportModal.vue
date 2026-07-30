<script setup lang="ts">
import { computed, reactive, ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import type { ArchiveAnalysis, ImportArchiveRequest } from "../../types";

const props = defineProps<{ agentId?: number; categoryId?: number }>();
const emit = defineEmits<{
  close: [];
  imported: [];
}>();

const archivePath = ref<string | null>(null);
const analysis = ref<ArchiveAnalysis | null>(null);
const isAnalyzing = ref(false);
const isImporting = ref(false);
const errorMessage = ref<string | null>(null);

const form = reactive({
  modName: "",
  description: "",
  author: "",
  selectedRoot: "",
});

const likelyRoots = computed(() => analysis.value?.entries.filter((e) => e.isLikelyModRoot) ?? []);

function fallbackNameFromPath(path: string): string {
  const filename = path.split(/[\\/]/).pop() ?? "New Mod";
  return filename.replace(/\.(zip|7z|rar)$/i, "");
}

async function pickArchive() {
  errorMessage.value = null;
  const path = await open({
    multiple: false,
    filters: [{ name: "Mod archive", extensions: ["zip", "7z", "rar"] }],
  });
  if (typeof path !== "string") return;

  archivePath.value = path;
  isAnalyzing.value = true;
  try {
    analysis.value = await invoke<ArchiveAnalysis>("analyze_archive", { archivePath: path });
    form.modName = analysis.value.deducedName ?? fallbackNameFromPath(path);
    form.author = analysis.value.deducedAuthor ?? "";
    form.selectedRoot = likelyRoots.value[0]?.path ?? "";
  } catch (e) {
    errorMessage.value = String(e);
    analysis.value = null;
  } finally {
    isAnalyzing.value = false;
  }
}

async function handleImport() {
  if (!archivePath.value || !form.modName.trim()) return;
  isImporting.value = true;
  errorMessage.value = null;
  try {
    const request: ImportArchiveRequest = {
      archivePath: archivePath.value,
      agentId: props.agentId ?? null,
      categoryId: props.categoryId ?? null,
      categoryItemId: null,
      selectedInternalRoot: form.selectedRoot || null,
      modName: form.modName.trim(),
      description: form.description.trim() || null,
      author: form.author.trim() || null,
    };
    await invoke("import_archive", { request });
    emit("imported");
  } catch (e) {
    errorMessage.value = String(e);
  } finally {
    isImporting.value = false;
  }
}
</script>

<template>
  <div class="modal-overlay" @click.self="emit('close')">
    <div class="modal-content card">
      <h2 class="settings-section-title">Import Mod</h2>

      <div v-if="!archivePath" class="form-group">
        <button type="button" class="btn btn-primary" @click="pickArchive">Choose Archive (.zip/.7z/.rar)</button>
      </div>

      <p v-if="isAnalyzing">Analyzing archive…</p>

      <form v-else-if="analysis" @submit.prevent="handleImport">
        <p class="settings-value">{{ archivePath }}</p>

        <div v-if="likelyRoots.length > 1" class="form-group">
          <label class="form-label" for="import-root">Which folder is the mod?</label>
          <select id="import-root" v-model="form.selectedRoot" class="form-input">
            <option v-for="root in likelyRoots" :key="root.path" :value="root.path">{{ root.path }}</option>
          </select>
        </div>

        <div class="form-group">
          <label class="form-label" for="import-name">Name</label>
          <input id="import-name" v-model="form.modName" class="form-input" type="text" required />
        </div>
        <div class="form-group">
          <label class="form-label" for="import-description">Description</label>
          <textarea id="import-description" v-model="form.description" class="form-input" rows="3"></textarea>
        </div>
        <div class="form-group">
          <label class="form-label" for="import-author">Author</label>
          <input id="import-author" v-model="form.author" class="form-input" type="text" />
        </div>

        <p v-if="errorMessage" class="settings-error">{{ errorMessage }}</p>

        <div class="form-actions">
          <button type="button" class="btn btn-secondary" @click="emit('close')">Cancel</button>
          <button type="submit" class="btn btn-primary" :disabled="isImporting">
            {{ isImporting ? "Importing…" : "Import" }}
          </button>
        </div>
      </form>

      <template v-else>
        <p v-if="errorMessage" class="settings-error">{{ errorMessage }}</p>
        <div class="form-actions">
          <button type="button" class="btn btn-secondary" @click="emit('close')">Cancel</button>
        </div>
      </template>
    </div>
  </div>
</template>
