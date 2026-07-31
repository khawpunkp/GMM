<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue';
import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import { PhPencilSimple, PhX } from '@phosphor-icons/vue';
import VueButton from '@/components/ui/button/VueButton.vue';
import VueInput from '@/components/ui/input/VueInput.vue';
import Label from '@/components/ui/input/Label.vue';
import { VueSelect } from '@/components/ui/select';
import VueTypography from '@/components/ui/typography/VueTypography.vue';
import type { Agent, AgentDetails, AgentInput } from '../../types';
import {
   ATTRIBUTE_ICONS,
   parseAgentDetails,
   RANK_ICONS,
   resolveAgentImageSrc,
   serializeAgentDetails,
   SPECIALITY_ICONS,
} from '../../utils/agent';

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

const toSelectOptions = (opts: string[]) => opts.map((opt) => ({ label: opt, value: opt }));
const rankSelectOptions = toSelectOptions(RANK_OPTIONS);
const attributeSelectOptions = toSelectOptions(ATTRIBUTE_OPTIONS);
const specialitySelectOptions = toSelectOptions(SPECIALITY_OPTIONS);

// Creating an agent drops straight into the form; an existing one opens read-only.
const isEditing = ref(props.initialAgent === undefined);
// A built-in agent's name/image/stats come from definitions/zzz.toml and are rewritten on every
// version-gated re-sync, so edits here wouldn't survive an update. Aliases are the one field
// that sync treats as additive-only, so they stay editable.
const canEditDetails = computed(() => !props.initialAgent?.isBuiltin);

const name = ref('');
const baseImage = ref<string | null>(null);
const aliases = reactive<string[]>([]);
const aliasInput = ref('');
const details = reactive<AgentDetails>({ rank: '', attribute: '', speciality: '' });

function resetFromAgent() {
   const agent = props.initialAgent;
   name.value = agent?.name ?? '';
   baseImage.value = agent?.baseImage ?? null;
   aliases.splice(0, aliases.length, ...(agent?.aliases ?? []));
   Object.assign(details, parseAgentDetails(agent?.details ?? null));
   aliasInput.value = '';
}
resetFromAgent();

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

const statRows = computed(() => [
   { label: 'Rank', value: details.rank, icon: RANK_ICONS[details.rank] },
   { label: 'Attribute', value: details.attribute, icon: ATTRIBUTE_ICONS[details.attribute] },
   { label: 'Speciality', value: details.speciality, icon: SPECIALITY_ICONS[details.speciality] },
]);

// Fires both on agent-to-agent navigation and after a save (the parent reassigns the agent with
// the server's response) — either way, drop back to the read-only view.
watch(
   () => props.initialAgent,
   (agent) => {
      if (!agent) return;
      resetFromAgent();
      isEditing.value = false;
   },
);

function cancelEdit() {
   resetFromAgent();
   isEditing.value = false;
}

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
   <div v-auto-animate class="bg-card max-w-160 rounded-lg border border-white/10 p-6">
      <template v-if="!isEditing">
         <div class="mb-5 flex items-center gap-4">
            <img
               :src="resolveAgentImageSrc(baseImage)"
               alt=""
               class="bg-foreground size-20 rounded-lg border border-white/10 object-cover"
            />
            <div class="flex flex-col gap-1">
               <VueTypography variant="TitleB" as="h2">{{ name }}</VueTypography>
               <VueTypography
                  v-if="initialAgent?.isBuiltin"
                  variant="CaptionR"
                  as="span"
                  class="text-muted-foreground"
               >
                  Built-in agent
               </VueTypography>
            </div>
         </div>

         <div class="mb-4.5 flex flex-wrap gap-8">
            <div v-for="stat in statRows" :key="stat.label" class="flex flex-col gap-1.5">
               <Label>{{ stat.label }}</Label>
               <div class="flex items-center gap-2">
                  <img v-if="stat.icon" :src="stat.icon" alt="" class="size-5 object-contain" />
                  <VueTypography variant="BodyR" as="span">{{ stat.value || '—' }}</VueTypography>
               </div>
            </div>
         </div>

         <div class="mb-4.5 flex flex-col gap-2">
            <Label>Aliases</Label>
            <div v-auto-animate class="flex flex-wrap gap-2">
               <span
                  v-for="alias in aliases"
                  :key="alias"
                  class="bg-primary/15 rounded-full px-3 py-1 text-sm"
               >
                  {{ alias }}
               </span>
               <VueTypography
                  v-if="aliases.length === 0"
                  variant="CaptionR"
                  as="span"
                  class="text-muted-foreground"
               >
                  No aliases yet
               </VueTypography>
            </div>
         </div>

         <div class="mt-2.5 flex items-center justify-end gap-3">
            <slot name="actions" />
            <VueButton type="button" class="min-w-32" @click="isEditing = true">
               <PhPencilSimple :size="20" weight="fill" />
               Edit
            </VueButton>
         </div>
      </template>

      <form v-else @submit.prevent="handleSubmit">
         <div class="mb-5 flex items-center gap-4">
            <img
               :src="resolveAgentImageSrc(baseImage)"
               alt=""
               class="bg-foreground size-20 rounded-lg border border-white/10 object-cover"
            />
            <VueButton
               v-if="canEditDetails"
               type="button"
               variant="outlined"
               size="sm"
               @click="pickImage"
            >
               Choose Image
            </VueButton>
         </div>

         <VueTypography
            v-if="!canEditDetails"
            variant="CaptionR"
            as="p"
            class="text-muted-foreground mb-4.5"
         >
            This agent ships with the app — its name, image and stats are refreshed on every
            update, so only aliases can be changed here.
         </VueTypography>

         <VueInput
            id="agent-name"
            v-model="name"
            label="Name"
            container-class="mb-4.5"
            required
            :disabled="!canEditDetails"
         />

         <div class="mb-4.5 flex flex-wrap gap-4">
            <VueSelect
               v-model="rankModel"
               label="Rank"
               placeholder="—"
               clearable
               class="min-w-40 flex-1"
               :options="rankSelectOptions"
               :disabled="!canEditDetails"
            />
            <VueSelect
               v-model="attributeModel"
               label="Attribute"
               placeholder="—"
               clearable
               class="min-w-40 flex-1"
               :options="attributeSelectOptions"
               :disabled="!canEditDetails"
            />
            <VueSelect
               v-model="specialityModel"
               label="Speciality"
               placeholder="—"
               clearable
               class="min-w-40 flex-1"
               :options="specialitySelectOptions"
               :disabled="!canEditDetails"
            />
         </div>

         <div class="mb-4.5 flex flex-col gap-2">
            <Label>Aliases</Label>
            <div v-auto-animate class="mb-2.5 flex flex-wrap gap-2">
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
            <VueButton
               v-if="initialAgent"
               type="button"
               variant="outlined"
               class="min-w-32"
               @click="cancelEdit"
            >
               Cancel
            </VueButton>
            <VueButton type="submit" class="min-w-32">{{ submitLabel }}</VueButton>
         </div>
      </form>
   </div>
</template>
