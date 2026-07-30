import { defineStore } from "pinia";
import { invoke } from "@tauri-apps/api/core";
import type { ModGroup } from "../types";

export const useModGroupsStore = defineStore("modGroups", {
  state: () => ({
    groups: [] as ModGroup[],
    isLoading: false,
  }),
  actions: {
    async fetchAll() {
      this.isLoading = true;
      try {
        this.groups = await invoke<ModGroup[]>("list_mod_groups");
      } finally {
        this.isLoading = false;
      }
    },
    async create(name: string, modIds: number[]) {
      const group = await invoke<ModGroup>("create_mod_group", { name, modIds });
      this.groups.push(group);
      return group;
    },
    async toggle(groupId: number) {
      const isEnabled = await invoke<boolean>("toggle_mod_group", { groupId });
      const group = this.groups.find((g) => g.id === groupId);
      if (group) {
        group.isEnabled = isEnabled;
        group.members.forEach((m) => (m.isEnabled = isEnabled));
      }
      return isEnabled;
    },
    async rename(groupId: number, name: string) {
      await invoke("rename_mod_group", { groupId, name });
      const group = this.groups.find((g) => g.id === groupId);
      if (group) group.name = name;
    },
    async removeMember(groupId: number, modId: number) {
      const updated = await invoke<ModGroup | null>("remove_mod_from_group", { groupId, modId });
      if (updated) {
        const index = this.groups.findIndex((g) => g.id === groupId);
        if (index !== -1) this.groups[index] = updated;
      } else {
        this.groups = this.groups.filter((g) => g.id !== groupId);
      }
      return updated;
    },
    async disband(groupId: number) {
      await invoke("delete_mod_group", { groupId });
      this.groups = this.groups.filter((g) => g.id !== groupId);
    },
  },
});
