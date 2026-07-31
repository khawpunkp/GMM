<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import VueButton from '@/components/ui/button/VueButton.vue';
import VueInput from '@/components/ui/input/VueInput.vue';
import Label from '@/components/ui/input/Label.vue';
import { VueSelect } from '@/components/ui/select';
import VueTypography from '@/components/ui/typography/VueTypography.vue';
import type { KeybindInfo, PersistVar } from '../../types';

const props = defineProps<{ modId: number }>();
const emit = defineEmits<{ close: [] }>();

const keybinds = ref<KeybindInfo[]>([]);
const persistVars = ref<PersistVar[]>([]);
const isLoading = ref(true);
const errorMessage = ref<string | null>(null);

const savingVars = reactive<Record<string, boolean>>({});
const varErrors = reactive<Record<string, string>>({});

onMounted(async () => {
   try {
      const [kb, pv] = await Promise.all([
         invoke<KeybindInfo[]>('get_mod_keybinds', { modId: props.modId }),
         invoke<PersistVar[]>('get_mod_persist_vars', { modId: props.modId }),
      ]);
      keybinds.value = kb;
      persistVars.value = pv;
   } catch (e) {
      errorMessage.value = String(e);
   } finally {
      isLoading.value = false;
   }
});

async function saveVar(varName: string, rawValue: string) {
   const value = Number(rawValue);
   if (!Number.isInteger(value)) return;

   savingVars[varName] = true;
   varErrors[varName] = '';
   try {
      await invoke('set_mod_persist_var', { modId: props.modId, varName, value });
      const target = persistVars.value.find((v) => v.name === varName);
      if (target) target.value = value;
   } catch (e) {
      varErrors[varName] = String(e);
   } finally {
      savingVars[varName] = false;
   }
}
</script>

<template>
   <div
      class="fixed inset-0 z-100 flex items-center justify-center bg-black/60"
      @click.self="emit('close')"
   >
      <div
         class="bg-card max-h-[85vh] w-11/12 max-w-120 overflow-y-auto rounded-2xl border border-white/10 p-6"
      >
         <VueTypography variant="TitleB" as="h2" class="mb-2.5">Keybinds</VueTypography>

         <template v-if="errorMessage">
            <VueTypography variant="CaptionR" as="p" class="text-destructive">
               {{ errorMessage }}
            </VueTypography>
         </template>
         <template v-else>
            <VueTypography
               v-if="keybinds.length === 0"
               variant="CaptionR"
               as="p"
               class="text-muted-foreground"
            >
               No keybinds found — this mod's INI has no "; Constants" section, or none of its
               [Key...] sections have a value set yet.
            </VueTypography>
            <ul v-else class="mt-2.5 mb-5 flex flex-col gap-2">
               <li
                  v-for="kb in keybinds"
                  :key="kb.title"
                  class="flex items-center justify-between rounded-md bg-white/5 px-3 py-2 text-[13px]"
               >
                  <span>{{ kb.title }}</span>
                  <span class="rounded bg-black/30 px-2 py-0.5 font-mono">{{ kb.key }}</span>
               </li>
            </ul>

            <template v-if="persistVars.length > 0">
               <VueTypography variant="TitleB" as="h2" class="mb-2.5">Toggle memory</VueTypography>
               <div v-for="pv in persistVars" :key="pv.name" class="mb-4.5">
                  <VueSelect
                     v-if="pv.options.length > 0"
                     :model-value="pv.value"
                     :label="pv.name"
                     :options="pv.options.map((opt) => ({ label: String(opt), value: opt }))"
                     :disabled="savingVars[pv.name]"
                     @update:model-value="(value) => saveVar(pv.name, String(value))"
                  />
                  <template v-else>
                     <Label :for="`persist-${pv.name}`">{{ pv.name }}</Label>
                     <VueInput
                        :id="`persist-${pv.name}`"
                        type="number"
                        :model-value="pv.value"
                        :disabled="savingVars[pv.name]"
                        @change="saveVar(pv.name, ($event.target as HTMLInputElement).value)"
                     />
                  </template>
                  <VueTypography
                     v-if="varErrors[pv.name]"
                     variant="CaptionR"
                     as="p"
                     class="text-destructive mt-1.5"
                  >
                     {{ varErrors[pv.name] }}
                  </VueTypography>
               </div>
            </template>
         </template>

         <div class="flex items-center justify-end">
            <VueButton type="button" variant="outlined" @click="emit('close')">Close</VueButton>
         </div>
      </div>
   </div>
</template>
