<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import AgentForm from "../../components/agents/AgentForm.vue";
import ModCard from "../../components/mods/ModCard.vue";
import GroupCard from "../../components/mods/GroupCard.vue";
import ModEditModal from "../../components/mods/ModEditModal.vue";
import ImportModal from "../../components/mods/ImportModal.vue";
import KeybindsPopup from "../../components/mods/KeybindsPopup.vue";
import { useAgentsStore } from "../../stores/agents";
import { useModsStore } from "../../stores/mods";
import { useModGroupsStore } from "../../stores/modGroups";
import type { Agent, AgentInput, Mod, ModInput } from "../../types";

const route = useRoute("/agents/[slug]");
const router = useRouter();
const agentsStore = useAgentsStore();
const modsStore = useModsStore();
const modGroupsStore = useModGroupsStore();

const agent = ref<Agent | null>(null);
const isLoading = ref(true);
const errorMessage = ref<string | null>(null);
const editingMod = ref<Mod | null>(null);
const keybindsMod = ref<Mod | null>(null);
const isImporting = ref(false);

const isSelecting = ref(false);
const selectedModIds = ref<Set<number>>(new Set());

const ungroupedMods = computed(() => modsStore.mods.filter((m) => m.groupId === null));
const visibleGroups = computed(() => {
  const groupedIds = new Set(modsStore.mods.map((m) => m.groupId).filter((id): id is number => id !== null));
  return modGroupsStore.groups.filter((g) => groupedIds.has(g.id));
});

async function loadMods() {
  if (!agent.value) return;
  await Promise.all([modsStore.fetchByAgent(agent.value.id), modGroupsStore.fetchAll()]);
}

onMounted(async () => {
  try {
    agent.value = await agentsStore.fetchOne(route.params.slug);
    await loadMods();
  } catch (e) {
    errorMessage.value = String(e);
  } finally {
    isLoading.value = false;
  }
});

watch(
  () => route.params.slug,
  async (slug) => {
    if (!slug) return;
    agent.value = await agentsStore.fetchOne(slug);
    await loadMods();
  }
);

async function handleSubmit(input: AgentInput) {
  if (!agent.value) return;
  agent.value = await agentsStore.update(agent.value.slug, input);
}

async function handleDelete() {
  if (!agent.value || agent.value.isBuiltin) return;
  if (!confirm(`Delete "${agent.value.name}"? This cannot be undone.`)) return;
  await agentsStore.remove(agent.value.slug);
  router.push("/agents");
}

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
  await loadMods();
}

function toggleSelectMode() {
  isSelecting.value = !isSelecting.value;
  selectedModIds.value.clear();
}

function toggleSelect(mod: Mod) {
  if (selectedModIds.value.has(mod.id)) {
    selectedModIds.value.delete(mod.id);
  } else {
    selectedModIds.value.add(mod.id);
  }
}

async function groupSelected() {
  if (selectedModIds.value.size < 2) return;
  const name = prompt("Name this group:");
  if (!name || !name.trim()) return;
  try {
    await modGroupsStore.create(name.trim(), Array.from(selectedModIds.value));
    isSelecting.value = false;
    selectedModIds.value.clear();
    await loadMods();
  } catch (e) {
    alert(String(e));
  }
}
</script>

<template>
  <div>
    <div class="page-header">
      <h1 class="page-title"><i class="fa-solid fa-user"></i>{{ agent?.name ?? "Agent" }}</h1>
    </div>

    <p v-if="isLoading">Loading…</p>
    <p v-else-if="errorMessage">{{ errorMessage }}</p>
    <template v-else-if="agent">
      <AgentForm :key="agent.slug" :initial-agent="agent" submit-label="Save Changes" @submit="handleSubmit">
        <template #actions>
          <span v-if="agent.isBuiltin" class="builtin-note">Built-in agent — cannot be deleted</span>
          <button v-else type="button" class="btn btn-danger" @click="handleDelete">Delete</button>
        </template>
      </AgentForm>

      <div class="mods-section">
        <div class="page-header">
          <h2 class="settings-section-title">Mods</h2>
          <div class="form-actions settings-actions">
            <button
              v-if="isSelecting"
              type="button"
              class="btn btn-primary"
              :disabled="selectedModIds.size < 2"
              @click="groupSelected"
            >
              Group Selected ({{ selectedModIds.size }})
            </button>
            <button type="button" class="btn btn-secondary" @click="toggleSelectMode">
              {{ isSelecting ? "Cancel" : "Select Mods to Group" }}
            </button>
            <button type="button" class="btn btn-primary" @click="isImporting = true">+ Import Mod</button>
          </div>
        </div>
        <p v-if="modsStore.isLoading">Loading mods…</p>
        <p v-else-if="modsStore.mods.length === 0" class="settings-value">No mods for this agent yet.</p>
        <div v-else class="mod-grid">
          <GroupCard v-for="group in visibleGroups" :key="`group-${group.id}`" :group="group" />
          <ModCard
            v-for="mod in ungroupedMods"
            :key="mod.id"
            :mod="mod"
            :select-mode="isSelecting"
            :selected="selectedModIds.has(mod.id)"
            @edit="editingMod = $event"
            @delete="handleModDelete"
            @keybinds="keybindsMod = $event"
            @toggle-select="toggleSelect"
          />
        </div>
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
      v-if="isImporting && agent"
      :agent-id="agent.id"
      @imported="handleImported"
      @close="isImporting = false"
    />
  </div>
</template>
