import { defineStore } from "pinia";
import type { Mod, ModGroup } from "../types";

export const useModsStore = defineStore("mods", {
  state: () => ({
    mods: [] as Mod[],
    groups: [] as ModGroup[],
    isLoading: false,
  }),
});
