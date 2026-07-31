<script setup lang="ts">
import { computed } from "vue";
import { PhStack, PhCheckCircle } from "@phosphor-icons/vue";
import VueCard from "@/components/ui/card/VueCard.vue";
import VueTypography from "@/components/ui/typography/VueTypography.vue";
import {
  ATTRIBUTE_ICONS,
  parseAgentDetails,
  rankColor,
  resolveAgentImageSrc,
  SPECIALITY_ICONS,
} from "../../utils/agent";
import type { Agent } from "../../types";

const props = defineProps<{
  agent: Agent;
  totalMods: number;
  enabledMods: number;
}>();

const details = computed(() => parseAgentDetails(props.agent.details));
const barColor = computed(() => rankColor(details.value.rank));
const attributeIcon = computed(() =>
  details.value.attribute ? ATTRIBUTE_ICONS[details.value.attribute] : null,
);
const specialityIcon = computed(() =>
  details.value.speciality ? SPECIALITY_ICONS[details.value.speciality] : null,
);
</script>

<template>
  <RouterLink :to="`/agents/${agent.slug}`" class="block">
    <VueCard
      class="relative flex flex-col overflow-hidden p-0 no-underline transition-[transform,box-shadow] duration-300 hover:-translate-y-1.25 hover:shadow-[0_10px_25px_rgba(0,0,0,0.3)] hover:border-primary/30"
    >
      <div
        class="absolute top-2.5 right-2.5 flex flex-col items-end gap-1.5 z-20"
      >
        <span
          v-if="totalMods > 0"
          class="flex items-center gap-1.5 rounded-full bg-primary/85 px-2.5 py-1 text-xs font-bold text-white shadow-[0_1px_3px_rgba(0,0,0,0.3)]"
        >
          <PhStack :size="16" weight="fill" />{{ totalMods }}
        </span>
        <span
          v-if="enabledMods > 0"
          class="flex items-center gap-1.5 rounded-full bg-[rgba(29,209,161,0.9)] px-2.5 py-1 text-xs font-bold text-white shadow-[0_1px_3px_rgba(0,0,0,0.3)]"
        >
          <PhCheckCircle :size="16" weight="fill" />{{ enabledMods }}
        </span>
      </div>

      <div
        class="flex h-55 shrink-0 items-end justify-end gap-1.5 rounded-b-lg bg-cover bg-top p-2 z-10 relative bg-white overflow-hidden"
      >
        <img
          :src="resolveAgentImageSrc(agent.baseImage)"
          class="absolute size-full top-0 left-0 object-cover"
        />
        <img
          v-if="specialityIcon"
          :src="specialityIcon"
          alt=""
          class="size-8 rounded-full bg-[#1f1e36] object-contain p-1.5 z-10"
        />
        <img
          v-if="attributeIcon"
          :src="attributeIcon"
          alt=""
          class="size-8 rounded-full bg-[#1f1e36] object-contain p-1.5 z-10"
        />
      </div>
      <div
        class="-mt-4 h-10 rounded-b-lg"
        :style="{ backgroundColor: barColor }"
      />

      <div class="flex grow flex-col justify-center p-4 text-center">
        <VueTypography variant="BodyB" as="span">{{
          agent.name
        }}</VueTypography>
      </div>
    </VueCard>
  </RouterLink>
</template>
