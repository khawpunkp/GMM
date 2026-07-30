<script setup lang="ts">
import { computed } from "vue";
import VueButton from "@/components/ui/button/VueButton.vue";
import VueTypography from "@/components/ui/typography/VueTypography.vue";
import { useUpdaterStore } from "../stores/updater";

const emit = defineEmits<{ close: [] }>();
const updaterStore = useUpdaterStore();

const progressPercent = computed(() => {
  if (!updaterStore.totalBytes) return null;
  return Math.min(100, Math.round((updaterStore.downloadedBytes / updaterStore.totalBytes) * 100));
});

async function handleClose() {
  if (updaterStore.isDownloading) return;
  await updaterStore.dismiss();
  emit("close");
}

async function handleInstall() {
  await updaterStore.downloadAndInstall();
}

async function handleRestart() {
  await updaterStore.restart();
}
</script>

<template>
  <div class="fixed inset-0 z-100 flex items-center justify-center bg-black/60" @click.self="handleClose">
    <div class="w-11/12 max-w-120 max-h-[85vh] overflow-y-auto rounded-2xl border border-white/10 bg-card p-6">
      <VueTypography variant="TitleB" as="h2" class="mb-2.5">Update Available: v{{ updaterStore.update?.version }}</VueTypography>
      <VueTypography variant="CaptionR" as="p" class="mb-4 text-muted-foreground">
        Currently running v{{ updaterStore.update?.currentVersion }}
      </VueTypography>
      <VueTypography v-if="updaterStore.update?.body" variant="CaptionR" as="p" class="mb-5 max-h-50 overflow-y-auto whitespace-pre-wrap text-foreground/70">
        {{ updaterStore.update.body }}
      </VueTypography>

      <div v-if="updaterStore.isDownloading" class="my-2.5">
        <div class="h-2 w-full overflow-hidden rounded-full bg-white/10">
          <div class="h-full bg-primary transition-[width]" :style="{ width: (progressPercent ?? 0) + '%' }"></div>
        </div>
        <VueTypography variant="CaptionR" as="p" class="mt-3.5 text-muted-foreground">
          {{ progressPercent !== null ? `${progressPercent}%` : "Downloading…" }}
        </VueTypography>
      </div>

      <VueTypography v-if="updaterStore.errorMessage" variant="CaptionR" as="p" class="mt-3.5 text-destructive">
        {{ updaterStore.errorMessage }}
      </VueTypography>

      <div class="mt-2.5 flex items-center justify-end gap-3">
        <VueButton v-if="updaterStore.isReadyToRestart" type="button" @click="handleRestart">Restart Now</VueButton>
        <template v-else>
          <VueButton type="button" variant="outlined" :disabled="updaterStore.isDownloading" @click="handleClose">Later</VueButton>
          <VueButton type="button" :disabled="updaterStore.isDownloading" @click="handleInstall">
            {{ updaterStore.isDownloading ? "Downloading…" : "Download & Install" }}
          </VueButton>
        </template>
      </div>
    </div>
  </div>
</template>
