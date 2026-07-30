<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import AgentForm from "../../components/agents/AgentForm.vue";
import ModCard from "../../components/mods/ModCard.vue";
import ModEditModal from "../../components/mods/ModEditModal.vue";
import ImportModal from "../../components/mods/ImportModal.vue";
import KeybindsPopup from "../../components/mods/KeybindsPopup.vue";
import { useAgentsStore } from "../../stores/agents";
import { useModsStore } from "../../stores/mods";
import type { Agent, AgentInput, Mod, ModInput } from "../../types";

const route = useRoute("/agents/[slug]");
const router = useRouter();
const agentsStore = useAgentsStore();
const modsStore = useModsStore();

const agent = ref<Agent | null>(null);
const isLoading = ref(true);
const errorMessage = ref<string | null>(null);
const editingMod = ref<Mod | null>(null);
const keybindsMod = ref<Mod | null>(null);
const isImporting = ref(false);

onMounted(async () => {
  try {
    agent.value = await agentsStore.fetchOne(route.params.slug);
    await modsStore.fetchByAgent(agent.value.id);
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
    await modsStore.fetchByAgent(agent.value.id);
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

async function handleModDelete(mod: Mod) {
  if (!confirm(`Delete "${mod.name}"? This removes the mod folder from disk and cannot be undone.`)) return;
  await modsStore.remove(mod.id);
}

async function handleImported() {
  isImporting.value = false;
  if (agent.value) await modsStore.fetchByAgent(agent.value.id);
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
          <button type="button" class="btn btn-primary" @click="isImporting = true">+ Import Mod</button>
        </div>
        <p v-if="modsStore.isLoading">Loading mods…</p>
        <p v-else-if="modsStore.mods.length === 0" class="settings-value">No mods for this agent yet.</p>
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
      </div>
    </template>

    <ModEditModal v-if="editingMod" :mod="editingMod" @submit="handleModSubmit" @close="editingMod = null" />

    <KeybindsPopup v-if="keybindsMod" :mod-id="keybindsMod.id" @close="keybindsMod = null" />

    <ImportModal
      v-if="isImporting && agent"
      :agent-id="agent.id"
      @imported="handleImported"
      @close="isImporting = false"
    />
  </div>
</template>
