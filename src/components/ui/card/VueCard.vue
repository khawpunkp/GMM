<template>
  <div
    data-slot="card"
    :class="cn('rounded-lg bg-card border border-white/10', props.class)"
  >
    <slot />
  </div>
</template>

<script setup lang="ts">
import type { HTMLAttributes } from "vue";
import { cn } from "@/utils/general";

const props = defineProps<{
  class?: HTMLAttributes["class"];
}>();
</script>
