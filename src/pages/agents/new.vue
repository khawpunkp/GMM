<script setup lang="ts">
import { useRouter } from "vue-router";
import AgentForm from "../../components/agents/AgentForm.vue";
import { useAgentsStore } from "../../stores/agents";
import type { AgentInput } from "../../types";

const router = useRouter();
const agentsStore = useAgentsStore();

async function handleSubmit(input: AgentInput) {
  const agent = await agentsStore.create(input);
  router.push(`/agents/${agent.slug}`);
}
</script>

<template>
  <div>
    <div class="page-header">
      <h1 class="page-title"><i class="fa-solid fa-user-plus"></i>Add Agent</h1>
    </div>
    <AgentForm submit-label="Create Agent" @submit="handleSubmit" />
  </div>
</template>
