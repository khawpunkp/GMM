<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import { useRoute } from "vue-router";
import ModCard from "../../components/mods/ModCard.vue";
import ModEditModal from "../../components/mods/ModEditModal.vue";
import ImportModal from "../../components/mods/ImportModal.vue";
import KeybindsPopup from "../../components/mods/KeybindsPopup.vue";
import { useCategoriesStore } from "../../stores/categories";
import { useModsStore } from "../../stores/mods";
import type { Category, Mod, ModInput } from "../../types";

const route = useRoute("/categories/[slug]");
const categoriesStore = useCategoriesStore();
const modsStore = useModsStore();

const category = ref<Category | null>(null);
const isLoading = ref(true);
const errorMessage = ref<string | null>(null);
const editingMod = ref<Mod | null>(null);
const keybindsMod = ref<Mod | null>(null);
const isImporting = ref(false);

async function loadForSlug(slug: string) {
  isLoading.value = true;
  errorMessage.value = null;
  try {
    if (categoriesStore.categories.length === 0) await categoriesStore.fetchAll();
    const found = categoriesStore.bySlug(slug);
    if (!found) throw new Error(`Unknown category: ${slug}`);
    category.value = found;
    await modsStore.fetchByCategory(found.id);
  } catch (e) {
    errorMessage.value = String(e);
  } finally {
    isLoading.value = false;
  }
}

onMounted(() => loadForSlug(route.params.slug));

watch(
  () => route.params.slug,
  (slug) => {
    if (slug) loadForSlug(slug);
  }
);

async function handleModSubmit(input: ModInput) {
  if (!editingMod.value) return;
  await modsStore.update(editingMod.value.id, input);
  editingMod.value = null;
}

async function handleModRecategorize(target: { agentId?: number; categoryId?: number }) {
  if (!editingMod.value) return;
  await modsStore.updateCategory(editingMod.value.id, target);
  editingMod.value = null;
}

async function handleModDelete(mod: Mod) {
  if (!confirm(`Delete "${mod.name}"? This removes the mod folder from disk and cannot be undone.`)) return;
  await modsStore.remove(mod.id);
}

async function handleImported() {
  isImporting.value = false;
  if (category.value) await modsStore.fetchByCategory(category.value.id);
}
</script>

<template>
  <div>
    <div class="page-header">
      <h1 class="page-title"><i class="fa-solid fa-shapes"></i>{{ category?.name ?? "Category" }}</h1>
      <button v-if="category" type="button" class="btn btn-primary" @click="isImporting = true">+ Import Mod</button>
    </div>

    <p v-if="isLoading">Loading…</p>
    <p v-else-if="errorMessage" class="settings-error">{{ errorMessage }}</p>
    <template v-else>
      <p v-if="modsStore.mods.length === 0" class="settings-value">No mods in this category yet.</p>
      <div v-else class="mod-grid">
        <ModCard
          v-for="mod in modsStore.mods"
          :key="mod.id"
          :mod="mod"
          @edit="editingMod = $event"
          @delete="handleModDelete"
          @keybinds="keybindsMod = $event"
        />
      </div>
    </template>

    <ModEditModal
      v-if="editingMod"
      :mod="editingMod"
      @submit="handleModSubmit"
      @recategorize="handleModRecategorize"
      @close="editingMod = null"
    />

    <KeybindsPopup v-if="keybindsMod" :mod-id="keybindsMod.id" @close="keybindsMod = null" />

    <ImportModal
      v-if="isImporting && category"
      :category-id="category.id"
      @imported="handleImported"
      @close="isImporting = false"
    />
  </div>
</template>
