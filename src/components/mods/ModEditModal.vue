<script setup lang="ts">
import { reactive, ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import type { Mod, ModInput } from "../../types";

const props = defineProps<{ mod: Mod }>();
const emit = defineEmits<{
  submit: [input: ModInput];
  close: [];
}>();

const form = reactive({
  name: props.mod.name,
  description: props.mod.description ?? "",
  author: props.mod.author ?? "",
});
const imageDataUrl = ref<string | null>(null);

async function pickImage() {
  const path = await open({
    multiple: false,
    filters: [{ name: "Images", extensions: ["png", "jpg", "jpeg", "webp", "gif"] }],
  });
  if (typeof path === "string") {
    imageDataUrl.value = await invoke<string>("read_image_as_data_url", { path });
  }
}

function handleSubmit() {
  emit("submit", {
    name: form.name.trim(),
    description: form.description.trim() || null,
    author: form.author.trim() || null,
    imageDataUrl: imageDataUrl.value,
  });
}
</script>

<template>
  <div class="modal-overlay" @click.self="emit('close')">
    <form class="modal-content card" @submit.prevent="handleSubmit">
      <h2 class="settings-section-title">Edit Mod</h2>

      <div class="agent-form-image">
        <img :src="imageDataUrl ?? '/images/placeholder.jpg'" alt="" class="agent-form-image-preview" />
        <button type="button" class="btn btn-secondary" @click="pickImage">Choose New Image</button>
      </div>

      <div class="form-group">
        <label class="form-label" for="mod-name">Name</label>
        <input id="mod-name" v-model="form.name" class="form-input" type="text" required />
      </div>
      <div class="form-group">
        <label class="form-label" for="mod-description">Description</label>
        <textarea id="mod-description" v-model="form.description" class="form-input" rows="3"></textarea>
      </div>
      <div class="form-group">
        <label class="form-label" for="mod-author">Author</label>
        <input id="mod-author" v-model="form.author" class="form-input" type="text" />
      </div>

      <div class="form-actions">
        <button type="button" class="btn btn-secondary" @click="emit('close')">Cancel</button>
        <button type="submit" class="btn btn-primary">Save</button>
      </div>
    </form>
  </div>
</template>
