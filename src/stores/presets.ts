import { defineStore } from "pinia";
import { invoke } from "@tauri-apps/api/core";
import type { Preset } from "../types";

export const usePresetsStore = defineStore("presets", {
  state: () => ({
    presets: [] as Preset[],
    isLoading: false,
  }),
  getters: {
    favorites: (state) => state.presets.filter((p) => p.isFavorite).slice(0, 3),
  },
  actions: {
    async fetchAll() {
      this.isLoading = true;
      try {
        this.presets = await invoke<Preset[]>("list_presets");
      } finally {
        this.isLoading = false;
      }
    },
    async create(name: string) {
      const preset = await invoke<Preset>("create_preset", { name });
      this.presets.push(preset);
      return preset;
    },
    async apply(presetId: number) {
      return invoke<string>("apply_preset", { presetId });
    },
    async overwrite(presetId: number) {
      await invoke("overwrite_preset", { presetId });
    },
    async remove(presetId: number) {
      await invoke("delete_preset", { presetId });
      this.presets = this.presets.filter((p) => p.id !== presetId);
    },
    async toggleFavorite(presetId: number, isFavorite: boolean) {
      await invoke("toggle_preset_favorite", { presetId, isFavorite });
      const preset = this.presets.find((p) => p.id === presetId);
      if (preset) preset.isFavorite = isFavorite;
    },
  },
});
