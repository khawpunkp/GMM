<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { PhMagnifyingGlass, PhCaretLeft } from "@phosphor-icons/vue";
import AgentForm from "../../../components/agents/AgentForm.vue";
import ModCard from "../../../components/mods/ModCard.vue";
import GroupCard from "../../../components/mods/GroupCard.vue";
import ModEditModal from "../../../components/mods/ModEditModal.vue";
import KeybindsPopup from "../../../components/mods/KeybindsPopup.vue";
import { quickImportMod } from "../../../utils/quickImport";
import VueButton from "@/components/ui/button/VueButton.vue";
import VueInput from "@/components/ui/input/VueInput.vue";
import { VueSelect } from "@/components/ui/select";
import VueTypography from "@/components/ui/typography/VueTypography.vue";
import { useAgentsStore } from "../../../stores/agents";
import { useModsStore } from "../../../stores/mods";
import { useModGroupsStore } from "../../../stores/modGroups";
import type { Agent, AgentInput, Mod, ModInput } from "../../../types";

const SORT_KEY_PREFIX = "sort_agent_";

const SORT_OPTIONS = [
  { label: "Name (A-Z)", value: "name-asc" },
  { label: "Name (Z-A)", value: "name-desc" },
  { label: "Date Added (Newest)", value: "date-desc" },
  { label: "Date Added (Oldest)", value: "date-asc" },
  { label: "Status (Enabled first)", value: "status-enabled" },
  { label: "Status (Disabled first)", value: "status-disabled" },
];

const route = useRoute("/agents/[slug]/");
const router = useRouter();
const agentsStore = useAgentsStore();
const modsStore = useModsStore();
const modGroupsStore = useModGroupsStore();

const agent = ref<Agent | null>(null);
const isLoading = ref(true);
const errorMessage = ref<string | null>(null);
const editingMod = ref<Mod | null>(null);
const keybindsMod = ref<Mod | null>(null);

const isSelecting = ref(false);
const selectedModIds = ref<Set<number>>(new Set());

const search = ref("");
const sortOption = ref("name-asc");

const ungroupedMods = computed(() =>
  modsStore.mods.filter((m) => m.groupId === null),
);
const visibleGroups = computed(() => {
  const groupedIds = new Set(
    modsStore.mods
      .map((m) => m.groupId)
      .filter((id): id is number => id !== null),
  );
  return modGroupsStore.groups.filter((g) => groupedIds.has(g.id));
});

const filteredGroups = computed(() => {
  const query = search.value.trim().toLowerCase();
  if (!query) return visibleGroups.value;
  return visibleGroups.value.filter((g) =>
    g.name.toLowerCase().includes(query),
  );
});

const sortedFilteredMods = computed(() => {
  const query = search.value.trim().toLowerCase();
  const filtered = ungroupedMods.value.filter((m) => {
    if (!query) return true;
    return (
      m.name.toLowerCase().includes(query) ||
      (m.author ?? "").toLowerCase().includes(query)
    );
  });

  return [...filtered].sort((a, b) => {
    switch (sortOption.value) {
      case "name-desc":
        return b.name.localeCompare(a.name);
      case "date-desc":
        return b.id - a.id;
      case "date-asc":
        return a.id - b.id;
      case "status-enabled":
        return Number(b.isEnabled) - Number(a.isEnabled);
      case "status-disabled":
        return Number(a.isEnabled) - Number(b.isEnabled);
      default:
        return a.name.localeCompare(b.name);
    }
  });
});

function loadSortOption(slug: string) {
  sortOption.value = localStorage.getItem(SORT_KEY_PREFIX + slug) ?? "name-asc";
}

watch(sortOption, (value) => {
  if (agent.value)
    localStorage.setItem(SORT_KEY_PREFIX + agent.value.slug, value);
});

async function loadMods() {
  if (!agent.value) return;
  await Promise.all([
    modsStore.fetchByAgent(agent.value.id),
    modGroupsStore.fetchAll(),
  ]);
}

onMounted(async () => {
  try {
    agent.value = await agentsStore.fetchOne(route.params.slug);
    loadSortOption(route.params.slug);
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
    search.value = "";
    agent.value = await agentsStore.fetchOne(slug);
    loadSortOption(slug);
    await loadMods();
  },
);

function goBack() {
  if (window.history.length > 1) {
    router.back();
  } else {
    router.push("/agents");
  }
}

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

async function handleModRecategorize(target: {
  agentId?: number;
  categoryId?: number;
}) {
  if (!editingMod.value) return;
  await modsStore.updateCategory(editingMod.value.id, target);
  editingMod.value = null;
}

async function handleModDelete(mod: Mod) {
  if (
    !confirm(
      `Delete "${mod.name}"? This removes the mod folder from disk and cannot be undone.`,
    )
  )
    return;
  await modsStore.remove(mod.id);
}

async function handleQuickImport() {
  if (!agent.value) return;
  const imported = await quickImportMod({ agentId: agent.value.id });
  if (imported) await loadMods();
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
    <div
      class="mb-6 flex flex-wrap items-center justify-between gap-5 border-b border-white/10 pb-4"
    >
      <VueTypography variant="H1B" as="h1" class="flex items-center gap-3">
        <button
          type="button"
          class="cursor-pointer p-0 text-foreground/70 hover:text-primary"
          title="Back to list"
          @click="goBack"
        >
          <PhCaretLeft :size="32" />
        </button>
        {{ agent?.name ?? "Agent" }}
      </VueTypography>
    </div>

    <p v-if="errorMessage">{{ errorMessage }}</p>
    <template v-else-if="agent">
      <AgentForm
        :key="agent.slug"
        :initial-agent="agent"
        submit-label="Save Changes"
        @submit="handleSubmit"
      >
        <template #actions>
          <VueTypography
            v-if="agent.isBuiltin"
            variant="CaptionR"
            as="span"
            class="text-muted-foreground"
          >
            Built-in agent — cannot be deleted
          </VueTypography>
          <VueButton
            v-else
            type="button"
            variant="outlined"
            color="error"
            @click="handleDelete"
            >Delete</VueButton
          >
        </template>
      </AgentForm>

      <div class="mt-8">
        <div
          class="mb-6 flex flex-wrap items-center justify-between gap-5 border-b border-white/10 pb-4"
        >
          <VueTypography variant="TitleB" as="h2">Mods</VueTypography>
          <div class="flex items-center gap-3">
            <VueButton
              v-if="isSelecting"
              type="button"
              :disabled="selectedModIds.size < 2"
              @click="groupSelected"
            >
              Group Selected ({{ selectedModIds.size }})
            </VueButton>
            <VueButton
              type="button"
              variant="outlined"
              @click="toggleSelectMode"
            >
              {{ isSelecting ? "Cancel" : "Select Mods to Group" }}
            </VueButton>
            <VueButton type="button" @click="handleQuickImport"
              >+ Import Mod</VueButton
            >
          </div>
        </div>

        <div
          class="mb-6 flex flex-wrap items-center gap-5 border-b border-white/10 pb-4"
        >
          <VueInput
            v-model="search"
            container-class="ml-auto w-full max-w-75"
            placeholder="Search Mods..."
            label="Search"
          >
            <template #iconStart="{ color }"
              ><PhMagnifyingGlass :size="24" :color="color"
            /></template>
          </VueInput>
          <div class="w-full max-w-75">
            <VueSelect
              v-model="sortOption"
              :options="SORT_OPTIONS"
              label="Sort by"
            />
          </div>
        </div>

        <div
          v-if="modsStore.mods.length === 0"
          class="flex-1 flex justify-center items-center min-h-50"
        >
          <img src="/images/no-data.png" class="w-50" />
        </div>
        <p
          v-else-if="
            sortedFilteredMods.length === 0 && filteredGroups.length === 0
          "
          class="text-sm text-muted-foreground"
        >
          No mods match your search.
        </p>
        <div
          v-else
          class="grid gap-4"
          style="grid-template-columns: repeat(auto-fill, minmax(320px, 1fr))"
        >
          <GroupCard
            v-for="group in filteredGroups"
            :key="`group-${group.id}`"
            :group="group"
          />
          <ModCard
            v-for="mod in sortedFilteredMods"
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

    <KeybindsPopup
      v-if="keybindsMod"
      :mod-id="keybindsMod.id"
      @close="keybindsMod = null"
    />
  </div>
</template>
