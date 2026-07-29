import { defineStore } from "pinia";
import type { Preset } from "../types";

export const usePresetsStore = defineStore("presets", {
  state: () => ({
    presets: [] as Preset[],
    isLoading: false,
  }),
});
