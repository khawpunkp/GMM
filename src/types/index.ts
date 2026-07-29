export interface Agent {
  id: number;
  name: string;
  slug: string;
  description: string | null;
  details: string | null;
  baseImage: string | null;
  isBuiltin: boolean;
}

export interface Mod {
  id: number;
  agentId: number | null;
  categoryId: number | null;
  name: string;
  description: string | null;
  folderName: string;
  imageFilename: string | null;
  author: string | null;
}

export interface ModGroup {
  id: number;
  name: string;
  modIds: number[];
}

export interface PresetModEntry {
  modId: number;
  isEnabled: boolean;
}

export interface Preset {
  id: number;
  name: string;
  isFavorite: boolean;
  mods: PresetModEntry[];
}
