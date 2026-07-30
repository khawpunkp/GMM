<script setup lang="ts">
import { onMounted } from "vue";
import { useAgentsStore } from "../../stores/agents";
import { resolveAgentImageSrc } from "../../utils/agent";

const agentsStore = useAgentsStore();

onMounted(() => {
  agentsStore.fetchAll();
});
</script>

<template>
  <div>
    <div class="page-header">
      <h1 class="page-title"><i class="fa-solid fa-users"></i>Agents</h1>
      <RouterLink to="/agents/new" class="btn btn-primary">+ Add Agent</RouterLink>
    </div>

    <p v-if="agentsStore.isLoading">Loading…</p>
    <div v-else class="agent-grid">
      <RouterLink
        v-for="agent in agentsStore.agents"
        :key="agent.slug"
        :to="`/agents/${agent.slug}`"
        class="agent-card card"
      >
        <img :src="resolveAgentImageSrc(agent.baseImage)" alt="" class="agent-card-image" />
        <div class="agent-card-name">{{ agent.name }}</div>
        <span v-if="agent.isBuiltin" class="badge">Built-in</span>
      </RouterLink>
    </div>
  </div>
</template>
