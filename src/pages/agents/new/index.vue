<script setup lang="ts">
import { useRouter } from "vue-router";
import { PhCaretLeft } from "@phosphor-icons/vue";
import AgentForm from "../../../components/agents/AgentForm.vue";
import VueTypography from "@/components/ui/typography/VueTypography.vue";
import { useAgentsStore } from "../../../stores/agents";
import type { AgentInput } from "../../../types";

const router = useRouter();
const agentsStore = useAgentsStore();

async function handleSubmit(input: AgentInput) {
  const agent = await agentsStore.create(input);
  router.push(`/agents/${agent.slug}`);
}

function goBack() {
  if (window.history.length > 1) {
    router.back();
  } else {
    router.push("/agents");
  }
}
</script>

<template>
  <div>
    <div class="mb-6 flex items-center border-b border-white/10 pb-4">
      <VueTypography variant="H1B" as="h1" class="flex items-center gap-3">
        <button
          type="button"
          class="cursor-pointer p-0 text-foreground/70 hover:text-primary"
          title="Back to list"
          @click="goBack"
        >
          <PhCaretLeft :size="32" />
        </button>
        Add Agent
      </VueTypography>
    </div>
    <AgentForm submit-label="Create Agent" @submit="handleSubmit" />
  </div>
</template>
