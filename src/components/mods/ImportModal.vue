<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import VueButton from "@/components/ui/button/VueButton.vue";
import VueInput from "@/components/ui/input/VueInput.vue";
import Label from "@/components/ui/input/Label.vue";
import { VueSelect } from "@/components/ui/select";
import VueTypography from "@/components/ui/typography/VueTypography.vue";
import { useAgentsStore } from "../../stores/agents";
import { useCategoriesStore } from "../../stores/categories";
import type { ArchiveAnalysis, ImportArchiveRequest } from "../../types";

const props = defineProps<{ agentId?: number; categoryId?: number }>();
const emit = defineEmits<{
  close: [];
  imported: [];
}>();

const hasFixedTarget = props.agentId !== undefined || props.categoryId !== undefined;

const agentsStore = useAgentsStore();
const categoriesStore = useCategoriesStore();
const pickedTarget = ref("");

const targetOptions = computed(() => [
  ...agentsStore.agents.map((agent) => ({ label: `Character: ${agent.name}`, value: `agent:${agent.id}` })),
  ...categoriesStore.categories.map((category) => ({ label: `Category: ${category.name}`, value: `category:${category.id}` })),
]);

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
const rootOptions = computed(() => likelyRoots.value.map((root) => ({ label: root.path, value: root.path })));

onMounted(() => {
  if (hasFixedTarget) return;
  if (agentsStore.agents.length === 0) agentsStore.fetchAll();
  if (categoriesStore.categories.length === 0) categoriesStore.fetchAll();
});

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
  if (!hasFixedTarget && !pickedTarget.value) return;

  const [kind, idStr] = pickedTarget.value.split(":");
  const pickedId = Number(idStr);

  isImporting.value = true;
  errorMessage.value = null;
  try {
    const request: ImportArchiveRequest = {
      archivePath: archivePath.value,
      agentId: props.agentId ?? (kind === "agent" ? pickedId : null),
      categoryId: props.categoryId ?? (kind === "category" ? pickedId : null),
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
  <div class="fixed inset-0 z-100 flex items-center justify-center bg-black/60" @click.self="emit('close')">
    <div class="w-11/12 max-w-120 max-h-[85vh] overflow-y-auto rounded-2xl border border-white/10 bg-card p-6">
      <VueTypography variant="TitleB" as="h2" class="mb-2.5">Import Mod</VueTypography>

      <div v-if="!archivePath" class="mb-4.5">
        <VueButton type="button" @click="pickArchive">Choose Archive (.zip/.7z/.rar)</VueButton>
      </div>

      <p v-if="isAnalyzing">Analyzing archive…</p>

      <form v-else-if="analysis" @submit.prevent="handleImport">
        <VueTypography variant="CaptionR" as="p" class="mb-4 break-all text-muted-foreground">{{ archivePath }}</VueTypography>

        <div v-if="!hasFixedTarget" class="mb-4.5">
          <VueSelect v-model="pickedTarget" label="Import into" :options="targetOptions" placeholder="Choose a destination…" />
        </div>

        <div v-if="likelyRoots.length > 1" class="mb-4.5">
          <VueSelect v-model="form.selectedRoot" label="Which folder is the mod?" :options="rootOptions" />
        </div>

        <VueInput id="import-name" v-model="form.modName" label="Name" container-class="mb-4.5" required />
        <div class="mb-4.5 flex flex-col gap-2">
          <Label for="import-description">Description</Label>
          <textarea
            id="import-description"
            v-model="form.description"
            rows="3"
            class="rounded-2xl border border-white/10 bg-white/5 px-4 py-3 text-foreground outline-none transition-all focus:border-primary"
          ></textarea>
        </div>
        <VueInput id="import-author" v-model="form.author" label="Author" container-class="mb-4.5" />

        <VueTypography v-if="errorMessage" variant="CaptionR" as="p" class="mb-4 text-destructive">{{ errorMessage }}</VueTypography>

        <div class="flex items-center justify-end gap-3">
          <VueButton type="button" variant="outlined" @click="emit('close')">Cancel</VueButton>
          <VueButton type="submit" :disabled="isImporting || (!hasFixedTarget && !pickedTarget)">
            {{ isImporting ? "Importing…" : "Import" }}
          </VueButton>
        </div>
      </form>

      <template v-else>
        <VueTypography v-if="errorMessage" variant="CaptionR" as="p" class="mb-4 text-destructive">{{ errorMessage }}</VueTypography>
        <div class="flex items-center justify-end gap-3">
          <VueButton type="button" variant="outlined" @click="emit('close')">Cancel</VueButton>
        </div>
      </template>
    </div>
  </div>
</template>
