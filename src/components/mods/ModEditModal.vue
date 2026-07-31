<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue';
import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import VueButton from '@/components/ui/button/VueButton.vue';
import VueInput from '@/components/ui/input/VueInput.vue';
import { VueSelect } from '@/components/ui/select';
import VueTypography from '@/components/ui/typography/VueTypography.vue';
import { useAgentsStore } from '../../stores/agents';
import { useCategoriesStore } from '../../stores/categories';
import type { Mod, ModInput } from '../../types';

const props = defineProps<{ mod: Mod }>();
const emit = defineEmits<{
   submit: [input: ModInput];
   recategorize: [target: { agentId?: number; categoryId?: number }];
   close: [];
}>();

const agentsStore = useAgentsStore();
const categoriesStore = useCategoriesStore();

const form = reactive({
   name: props.mod.name,
   author: props.mod.author ?? '',
});
const imageDataUrl = ref<string | null>(null);

const currentTarget =
   props.mod.agentId !== null
      ? `agent:${props.mod.agentId}`
      : props.mod.categoryId !== null
        ? `category:${props.mod.categoryId}`
        : '';
const selectedTarget = ref(currentTarget);
const isMoving = ref(false);

const canMove = computed(
   () => selectedTarget.value !== '' && selectedTarget.value !== currentTarget,
);

const categoryOptions = computed(() => [
   ...agentsStore.agents.map((agent) => ({
      label: `Character: ${agent.name}`,
      value: `agent:${agent.id}`,
   })),
   ...categoriesStore.categories.map((category) => ({
      label: `Category: ${category.name}`,
      value: `category:${category.id}`,
   })),
]);

onMounted(() => {
   if (agentsStore.agents.length === 0) agentsStore.fetchAll();
   if (categoriesStore.categories.length === 0) categoriesStore.fetchAll();
});

async function pickImage() {
   const path = await open({
      multiple: false,
      filters: [{ name: 'Images', extensions: ['png', 'jpg', 'jpeg', 'webp', 'gif'] }],
   });
   if (typeof path === 'string') {
      imageDataUrl.value = await invoke<string>('read_image_as_data_url', { path });
   }
}

function handleSubmit() {
   emit('submit', {
      name: form.name.trim(),
      author: form.author.trim() || null,
      imageDataUrl: imageDataUrl.value,
   });
}

async function handleMove() {
   const [kind, idStr] = selectedTarget.value.split(':');
   const id = Number(idStr);
   isMoving.value = true;
   try {
      emit('recategorize', kind === 'agent' ? { agentId: id } : { categoryId: id });
   } finally {
      isMoving.value = false;
   }
}
</script>

<template>
   <div
      class="fixed inset-0 z-100 flex items-center justify-center bg-black/60"
      @click.self="emit('close')"
   >
      <form
         class="bg-card max-h-[85vh] w-11/12 max-w-120 overflow-y-auto rounded-2xl border border-white/10 p-6"
         @submit.prevent="handleSubmit"
      >
         <VueTypography variant="TitleB" as="h2" class="mb-2.5">Edit Mod</VueTypography>

         <div class="mb-5 flex flex-col items-center gap-4">
            <img
               :src="imageDataUrl ?? '/images/placeholder.jpg'"
               alt=""
               class="aspect-video w-full rounded-2xl border border-white/10 object-cover"
            />
            <VueButton type="button" variant="outlined" size="sm" @click="pickImage">
               Choose New Image
            </VueButton>
         </div>

         <VueInput
            id="mod-name"
            v-model="form.name"
            label="Name"
            container-class="mb-4.5"
            required
         />

         <VueInput id="mod-author" v-model="form.author" label="Author" container-class="mb-4.5" />

         <div class="mb-4.5">
            <VueSelect
               v-model="selectedTarget"
               label="Category"
               :options="categoryOptions"
               placeholder="Uncategorized"
               searchable
            />
            <div class="mt-2.5 flex items-center justify-start">
               <VueButton
                  type="button"
                  variant="outlined"
                  size="sm"
                  :disabled="!canMove || isMoving"
                  @click="handleMove"
               >
                  {{ isMoving ? 'Moving…' : 'Move to selected category' }}
               </VueButton>
            </div>
         </div>

         <div class="mt-2.5 flex items-center justify-end gap-3">
            <VueButton type="button" variant="outlined" @click="emit('close')">Cancel</VueButton>
            <VueButton type="submit">Save</VueButton>
         </div>
      </form>
   </div>
</template>
