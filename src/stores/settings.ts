import { defineStore } from "pinia";

export const useSettingsStore = defineStore("settings", {
  state: () => ({
    settings: {} as Record<string, string>,
    isLoading: false,
  }),
});
