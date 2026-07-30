<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import AgentForm from "../../components/agents/AgentForm.vue";
import { useAgentsStore } from "../../stores/agents";
import type { Agent, AgentInput } from "../../types";

const route = useRoute("/agents/[slug]");
const router = useRouter();
const agentsStore = useAgentsStore();

const agent = ref<Agent | null>(null);
const isLoading = ref(true);
const errorMessage = ref<string | null>(null);

onMounted(async () => {
  try {
    agent.value = await agentsStore.fetchOne(route.params.slug);
  } catch (e) {
    errorMessage.value = String(e);
  } finally {
    isLoading.value = false;
  }
});

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
</script>

<template>
  <div>
    <div class="page-header">
      <h1 class="page-title"><i class="fa-solid fa-user"></i>{{ agent?.name ?? "Agent" }}</h1>
    </div>

    <p v-if="isLoading">Loading…</p>
    <p v-else-if="errorMessage">{{ errorMessage }}</p>
    <AgentForm
      v-else-if="agent"
      :key="agent.slug"
      :initial-agent="agent"
      submit-label="Save Changes"
      @submit="handleSubmit"
    >
      <template #actions>
        <span v-if="agent.isBuiltin" class="builtin-note">Built-in agent — cannot be deleted</span>
        <button v-else type="button" class="btn btn-danger" @click="handleDelete">Delete</button>
      </template>
    </AgentForm>
  </div>
</template>
