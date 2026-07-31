<script setup lang="ts">
import { useRouter } from 'vue-router';
import { PhWarning } from '@phosphor-icons/vue';
import VueButton from '@/components/ui/button/VueButton.vue';
import VueTypography from '@/components/ui/typography/VueTypography.vue';

const props = defineProps<{
   missingModsFolder: boolean;
   missingGameExecutable: boolean;
}>();
const emit = defineEmits<{ close: [] }>();

const router = useRouter();

function goToSettings() {
   router.push('/settings');
   emit('close');
}
</script>

<template>
   <div class="fixed inset-0 z-100 flex items-center justify-center bg-black/60">
      <div
         class="bg-card max-h-[85vh] w-11/12 max-w-120 overflow-y-auto rounded-lg border border-white/10 p-6"
      >
         <VueTypography variant="TitleB" as="h2" class="mb-4 flex items-center gap-3">
            <PhWarning :size="28" weight="fill" class="text-accent" />
            Finish setting up
         </VueTypography>

         <VueTypography variant="BodyR" as="p" class="text-muted-foreground mb-4">
            Some paths aren't configured yet:
         </VueTypography>

         <ul class="mb-5 flex flex-col gap-2">
            <li
               v-if="props.missingModsFolder"
               class="flex flex-col gap-1 rounded-md bg-white/5 px-3 py-2"
            >
               <VueTypography variant="BodyB" as="span">Mods Folder</VueTypography>
               <VueTypography variant="CaptionR" as="span" class="text-muted-foreground">
                  Required — mods can't be listed, scanned or imported without it.
               </VueTypography>
            </li>
            <li
               v-if="props.missingGameExecutable"
               class="flex flex-col gap-1 rounded-md bg-white/5 px-3 py-2"
            >
               <VueTypography variant="BodyB" as="span">Game Executable</VueTypography>
               <VueTypography variant="CaptionR" as="span" class="text-muted-foreground">
                  Needed for Quick Launch.
               </VueTypography>
            </li>
         </ul>

         <div class="flex items-center justify-end">
            <VueButton type="button" class="min-w-32" @click="goToSettings">
               Go to Settings
            </VueButton>
         </div>
      </div>
   </div>
</template>
