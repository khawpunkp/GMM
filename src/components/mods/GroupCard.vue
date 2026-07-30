<script setup lang="ts">
import { ref } from "vue";
import {
  PhCaretDown,
  PhCaretRight,
  PhStack,
  PhPencilSimple,
  PhArrowsOut,
  PhX,
} from "@phosphor-icons/vue";
import VueCard from "@/components/ui/card/VueCard.vue";
import VueTypography from "@/components/ui/typography/VueTypography.vue";
import VueSwitch from "@/components/ui/switch/VueSwitch.vue";
import { useModGroupsStore } from "../../stores/modGroups";
import type { ModGroup } from "../../types";

const props = defineProps<{ group: ModGroup }>();

const modGroupsStore = useModGroupsStore();
const isExpanded = ref(false);
const isRenaming = ref(false);
const renameValue = ref(props.group.name);

function toggle() {
  modGroupsStore.toggle(props.group.id);
}

function startRename() {
  renameValue.value = props.group.name;
  isRenaming.value = true;
}

async function confirmRename() {
  if (renameValue.value.trim()) {
    await modGroupsStore.rename(props.group.id, renameValue.value.trim());
  }
  isRenaming.value = false;
}

async function removeMember(modId: number) {
  await modGroupsStore.removeMember(props.group.id, modId);
}

async function disband() {
  if (
    !confirm(
      `Ungroup "${props.group.name}"? The mods themselves won't be touched.`,
    )
  )
    return;
  await modGroupsStore.disband(props.group.id);
}
</script>

<template>
  <VueCard
    class="flex flex-col gap-2.5 border-primary/30 p-3.5 transition-opacity"
    :class="{ 'opacity-50': !group.isEnabled }"
  >
    <div class="flex items-center gap-2">
      <button
        type="button"
        class="cursor-pointer p-1 text-foreground/70 hover:opacity-100"
        title="Expand"
        @click="isExpanded = !isExpanded"
      >
        <PhCaretDown v-if="isExpanded" :size="16" />
        <PhCaretRight v-else :size="16" />
      </button>
      <PhStack :size="20" class="text-primary" weight="fill" />
      <input
        v-if="isRenaming"
        v-model="renameValue"
        class="grow rounded-md border border-white/10 bg-white/5 px-2 py-1 text-foreground outline-none"
        type="text"
        @keydown.enter="confirmRename"
        @blur="confirmRename"
      />
      <VueTypography
        v-else
        variant="BodyB"
        as="span"
        class="grow cursor-text"
        @dblclick="startRename"
      >
        {{ group.name }}
      </VueTypography>
      <span
        class="rounded-full bg-accent px-2 py-0.5 text-[11px] font-semibold text-background"
        >{{ group.members.length }}</span
      >
    </div>

    <div class="flex items-center gap-2">
      <VueSwitch
        :model-value="group.isEnabled"
        :title="group.isEnabled ? 'Enabled' : 'Disabled'"
        class="mr-auto"
        @update:model-value="toggle"
      />
      <button
        type="button"
        class="cursor-pointer p-1 text-foreground/70 hover:opacity-100"
        title="Rename"
        @click="startRename"
      >
        <PhPencilSimple :size="20" weight="fill" />
      </button>
      <button
        type="button"
        class="cursor-pointer p-1 text-foreground/70 hover:text-destructive"
        title="Ungroup"
        @click="disband"
      >
        <PhArrowsOut :size="20" weight="fill" color="#ff6b6b" />
      </button>
    </div>

    <ul
      v-if="isExpanded"
      class="mt-2 flex flex-col gap-1.5 border-t border-white/10 pt-2.5"
    >
      <li
        v-for="member in group.members"
        :key="member.modId"
        class="flex items-center justify-between text-[13px]"
      >
        <span :class="{ 'opacity-50': !member.isEnabled }">{{
          member.name
        }}</span>
        <button
          type="button"
          class="cursor-pointer p-1 text-foreground/70 hover:opacity-100"
          title="Remove from group"
          @click="removeMember(member.modId)"
        >
          <PhX :size="16" />
        </button>
      </li>
    </ul>
  </VueCard>
</template>
