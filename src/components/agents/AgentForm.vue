<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue';
import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import { PhX } from '@phosphor-icons/vue';
import VueButton from '@/components/ui/button/VueButton.vue';
import VueInput from '@/components/ui/input/VueInput.vue';
import Label from '@/components/ui/input/Label.vue';
import { VueSelect } from '@/components/ui/select';
import VueCheckbox from '@/components/ui/checkbox/VueCheckbox.vue';
import type { Agent, AgentDetails, AgentInput } from '../../types';
import { parseAgentDetails, resolveAgentImageSrc, serializeAgentDetails } from '../../utils/agent';

const props = defineProps<{
   initialAgent?: Agent;
   submitLabel: string;
}>();

const emit = defineEmits<{
   submit: [input: AgentInput];
}>();

// Reka UI's SelectItem forbids an empty-string value (that's reserved to mean "cleared, show the
// placeholder"), so the "unset" state isn't a selectable list item here — it's represented by
// `clearable` on each VueSelect below instead, via the rank/attribute/specialityModel proxies.
const RANK_OPTIONS = ['S', 'A'];
const ATTRIBUTE_OPTIONS = [
   'Electric',
   'Fire',
   'Ice',
   'Frost',
   'Ether',
   'Physical',
   'AuricInk',
   'HonedEdge',
   'Lumiflux',
];
const SPECIALITY_OPTIONS = ['Attack', 'Stun', 'Anomaly', 'Support', 'Defense', 'Rupture'];
const TYPE_OPTIONS = ['Slash', 'Strike', 'Pierce'];

const toSelectOptions = (opts: string[]) => opts.map((opt) => ({ label: opt, value: opt }));
const rankSelectOptions = toSelectOptions(RANK_OPTIONS);
const attributeSelectOptions = toSelectOptions(ATTRIBUTE_OPTIONS);
const specialitySelectOptions = toSelectOptions(SPECIALITY_OPTIONS);

const name = ref(props.initialAgent?.name ?? '');
const baseImage = ref<string | null>(props.initialAgent?.baseImage ?? null);
const aliases = reactive<string[]>([...(props.initialAgent?.aliases ?? [])]);
const aliasInput = ref('');
const details = reactive<AgentDetails>(parseAgentDetails(props.initialAgent?.details ?? null));

function detailModel(key: 'rank' | 'attribute' | 'speciality') {
   return computed({
      get: () => details[key] || undefined,
      set: (value) => {
         details[key] = value ?? '';
      },
   });
}
const rankModel = detailModel('rank');
const attributeModel = detailModel('attribute');
const specialityModel = detailModel('speciality');

watch(
   () => props.initialAgent,
   (agent) => {
      if (!agent) return;
      name.value = agent.name;
      baseImage.value = agent.baseImage;
      aliases.splice(0, aliases.length, ...agent.aliases);
      Object.assign(details, parseAgentDetails(agent.details));
   },
);

function addAlias() {
   const value = aliasInput.value.trim().toLowerCase();
   if (value && !aliases.includes(value)) {
      aliases.push(value);
   }
   aliasInput.value = '';
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
      filters: [{ name: 'Images', extensions: ['png', 'jpg', 'jpeg', 'webp', 'gif'] }],
   });
   if (typeof path === 'string') {
      baseImage.value = await invoke<string>('read_image_as_data_url', { path });
   }
}

function handleSubmit() {
   emit('submit', {
      name: name.value.trim(),
      details: serializeAgentDetails(details),
      baseImage: baseImage.value,
      aliases: [...aliases],
   });
}
</script>

<template>
   <form
      class="bg-card max-w-160 rounded-2xl border border-white/10 p-6"
      @submit.prevent="handleSubmit"
   >
      <div class="mb-5 flex items-center gap-4">
         <img
            :src="resolveAgentImageSrc(baseImage)"
            alt=""
            class="size-20 rounded-2xl border border-white/10 object-cover"
         />
         <VueButton type="button" variant="outlined" size="sm" @click="pickImage">
            Choose Image
         </VueButton>
      </div>

      <VueInput id="agent-name" v-model="name" label="Name" container-class="mb-4.5" required />

      <div class="mb-4.5 flex flex-wrap gap-4">
         <VueSelect
            v-model="rankModel"
            label="Rank"
            placeholder="—"
            clearable
            class="min-w-40 flex-1"
            :options="rankSelectOptions"
         />
         <VueSelect
            v-model="attributeModel"
            label="Attribute"
            placeholder="—"
            clearable
            class="min-w-40 flex-1"
            :options="attributeSelectOptions"
         />
         <VueSelect
            v-model="specialityModel"
            label="Speciality"
            placeholder="—"
            clearable
            class="min-w-40 flex-1"
            :options="specialitySelectOptions"
         />
      </div>

      <div class="mb-4.5 flex flex-col gap-2">
         <Label>Type</Label>
         <div class="flex flex-wrap gap-4">
            <label v-for="opt in TYPE_OPTIONS" :key="opt" class="flex items-center gap-2 text-sm">
               <VueCheckbox
                  :model-value="details.type.includes(opt)"
                  @update:model-value="() => toggleType(opt)"
               />
               {{ opt }}
            </label>
         </div>
      </div>

      <div class="mb-4.5 flex flex-col gap-2">
         <Label>Aliases</Label>
         <div class="mb-2.5 flex flex-wrap gap-2">
            <span
               v-for="alias in aliases"
               :key="alias"
               class="bg-primary/15 flex items-center gap-1.5 rounded-full py-1 pr-1.5 pl-3 text-sm"
            >
               {{ alias }}
               <button
                  type="button"
                  class="text-foreground/60 hover:text-destructive cursor-pointer p-1"
                  @click="removeAlias(alias)"
               >
                  <PhX :size="12" />
               </button>
            </span>
         </div>
         <div class="flex gap-2">
            <VueInput
               v-model="aliasInput"
               container-class="flex-1"
               placeholder="Add an alias…"
               @keydown.enter.prevent="addAlias"
            />
            <VueButton type="button" variant="outlined" @click="addAlias">Add</VueButton>
         </div>
      </div>

      <div class="mt-2.5 flex items-center justify-end gap-3">
         <slot name="actions" />
         <VueButton type="submit">{{ submitLabel }}</VueButton>
      </div>
   </form>
</template>
