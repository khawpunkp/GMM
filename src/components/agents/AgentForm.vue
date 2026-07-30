<script setup lang="ts">
import { reactive, ref, watch } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import type { Agent, AgentDetails, AgentInput } from "../../types";
import { parseAgentDetails, resolveAgentImageSrc, serializeAgentDetails } from "../../utils/agent";

const props = defineProps<{
  initialAgent?: Agent;
  submitLabel: string;
}>();

const emit = defineEmits<{
  submit: [input: AgentInput];
}>();

const RANK_OPTIONS = ["", "A", "S"];
const ATTRIBUTE_OPTIONS = ["", "Electric", "Fire", "Ice", "Frost", "Ether", "Physical", "AuricInk", "HonedEdge"];
const SPECIALITY_OPTIONS = ["", "Attack", "Stun", "Anomaly", "Support", "Defense", "Rupture"];
const TYPE_OPTIONS = ["Slash", "Strike", "Pierce"];

const name = ref(props.initialAgent?.name ?? "");
const description = ref(props.initialAgent?.description ?? "");
const baseImage = ref<string | null>(props.initialAgent?.baseImage ?? null);
const aliases = reactive<string[]>([...(props.initialAgent?.aliases ?? [])]);
const aliasInput = ref("");
const details = reactive<AgentDetails>(parseAgentDetails(props.initialAgent?.details ?? null));

watch(
  () => props.initialAgent,
  (agent) => {
    if (!agent) return;
    name.value = agent.name;
    description.value = agent.description ?? "";
    baseImage.value = agent.baseImage;
    aliases.splice(0, aliases.length, ...agent.aliases);
    Object.assign(details, parseAgentDetails(agent.details));
  }
);

function addAlias() {
  const value = aliasInput.value.trim().toLowerCase();
  if (value && !aliases.includes(value)) {
    aliases.push(value);
  }
  aliasInput.value = "";
}

function removeAlias(alias: string) {
  const index = aliases.indexOf(alias);
  if (index !== -1) aliases.splice(index, 1);
}

function toggleType(type: string) {
  const index = details.type.indexOf(type);
  if (index === -1) {
    details.type.push(type);
  } else {
    details.type.splice(index, 1);
  }
}

async function pickImage() {
  const path = await open({
    multiple: false,
    filters: [{ name: "Images", extensions: ["png", "jpg", "jpeg", "webp", "gif"] }],
  });
  if (typeof path === "string") {
    baseImage.value = await invoke<string>("read_image_as_data_url", { path });
  }
}

function handleSubmit() {
  emit("submit", {
    name: name.value.trim(),
    description: description.value.trim() || null,
    details: serializeAgentDetails(details),
    baseImage: baseImage.value,
    aliases: [...aliases],
  });
}
</script>

<template>
  <form class="agent-form card" @submit.prevent="handleSubmit">
    <div class="agent-form-image">
      <img :src="resolveAgentImageSrc(baseImage)" alt="" class="agent-form-image-preview" />
      <button type="button" class="btn btn-secondary" @click="pickImage">Choose Image</button>
    </div>

    <div class="form-group">
      <label class="form-label" for="agent-name">Name</label>
      <input id="agent-name" v-model="name" class="form-input" type="text" required />
    </div>

    <div class="form-group">
      <label class="form-label" for="agent-description">Description</label>
      <textarea id="agent-description" v-model="description" class="form-input" rows="3"></textarea>
    </div>

    <div class="form-row">
      <div class="form-group">
        <label class="form-label" for="agent-rank">Rank</label>
        <select id="agent-rank" v-model="details.rank" class="form-input">
          <option v-for="opt in RANK_OPTIONS" :key="opt" :value="opt">{{ opt || "—" }}</option>
        </select>
      </div>

      <div class="form-group">
        <label class="form-label" for="agent-attribute">Attribute</label>
        <select id="agent-attribute" v-model="details.attribute" class="form-input">
          <option v-for="opt in ATTRIBUTE_OPTIONS" :key="opt" :value="opt">{{ opt || "—" }}</option>
        </select>
      </div>

      <div class="form-group">
        <label class="form-label" for="agent-speciality">Speciality</label>
        <select id="agent-speciality" v-model="details.speciality" class="form-input">
          <option v-for="opt in SPECIALITY_OPTIONS" :key="opt" :value="opt">{{ opt || "—" }}</option>
        </select>
      </div>
    </div>

    <div class="form-group">
      <span class="form-label">Type</span>
      <div class="checkbox-row">
        <label v-for="opt in TYPE_OPTIONS" :key="opt" class="checkbox-label">
          <input type="checkbox" :checked="details.type.includes(opt)" @change="toggleType(opt)" />
          {{ opt }}
        </label>
      </div>
    </div>

    <div class="form-group">
      <span class="form-label">Aliases</span>
      <div class="chip-list">
        <span v-for="alias in aliases" :key="alias" class="chip">
          {{ alias }}
          <button type="button" class="chip-remove" @click="removeAlias(alias)">×</button>
        </span>
      </div>
      <div class="chip-input-row">
        <input
          v-model="aliasInput"
          class="form-input"
          type="text"
          placeholder="Add an alias…"
          @keydown.enter.prevent="addAlias"
        />
        <button type="button" class="btn btn-secondary" @click="addAlias">Add</button>
      </div>
    </div>

    <div class="form-actions">
      <slot name="actions" />
      <button type="submit" class="btn btn-primary">{{ submitLabel }}</button>
    </div>
  </form>
</template>
