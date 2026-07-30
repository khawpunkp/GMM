<script setup lang="ts">
import { ref } from "vue";
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
  if (!confirm(`Ungroup "${props.group.name}"? The mods themselves won't be touched.`)) return;
  await modGroupsStore.disband(props.group.id);
}
</script>

<template>
  <div class="card mod-card group-card" :class="{ 'mod-card-disabled': !group.isEnabled }">
    <div class="group-card-header">
      <button type="button" class="icon-btn" title="Expand" @click="isExpanded = !isExpanded">
        <i class="fa-solid" :class="isExpanded ? 'fa-chevron-down' : 'fa-chevron-right'"></i>
      </button>
      <i class="fa-solid fa-layer-group group-card-icon"></i>
      <input
        v-if="isRenaming"
        v-model="renameValue"
        class="form-input group-rename-input"
        type="text"
        @keydown.enter="confirmRename"
        @blur="confirmRename"
      />
      <span v-else class="mod-card-name group-card-name" @dblclick="startRename">{{ group.name }}</span>
      <span class="badge group-card-count">{{ group.members.length }}</span>
    </div>

    <div class="mod-card-actions">
      <label class="switch" :title="group.isEnabled ? 'Enabled' : 'Disabled'">
        <input type="checkbox" :checked="group.isEnabled" @change="toggle" />
        <span class="switch-slider"></span>
      </label>
      <button type="button" class="icon-btn" title="Rename" @click="startRename">
        <i class="fa-solid fa-pen"></i>
      </button>
      <button type="button" class="icon-btn icon-btn-danger" title="Ungroup" @click="disband">
        <i class="fa-solid fa-object-ungroup"></i>
      </button>
    </div>

    <ul v-if="isExpanded" class="group-member-list">
      <li v-for="member in group.members" :key="member.modId" class="group-member-row">
        <span :class="{ 'group-member-disabled': !member.isEnabled }">{{ member.name }}</span>
        <button type="button" class="icon-btn" title="Remove from group" @click="removeMember(member.modId)">
          <i class="fa-solid fa-xmark"></i>
        </button>
      </li>
    </ul>
  </div>
</template>
