import { defineStore } from "pinia";
import type { Agent } from "../types";

export const useAgentsStore = defineStore("agents", {
  state: () => ({
    agents: [] as Agent[],
    isLoading: false,
  }),
});
