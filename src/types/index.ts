export interface Agent {
  id: number;
  name: string;
  slug: string;
  description: string | null;
  details: string | null;
  baseImage: string | null;
  isBuiltin: boolean;
  aliases: string[];
}

// Structured shape serialized into Agent.details (a free-form TEXT column in the schema).
export interface AgentDetails {
  rank: string;
  attribute: string;
  speciality: string;
  type: string[];
}

export interface AgentInput {
  name: string;
  description: string | null;
  details: string | null;
  baseImage: string | null;
  aliases: string[];
}

export interface Mod {
  id: number;
  agentId: number | null;
  categoryId: number | null;
  categoryItemId: number | null;
  name: string;
  description: string | null;
  folderName: string;
  imageFilename: string | null;
  author: string | null;
  isEnabled: boolean;
}

export interface ModInput {
  name: string;
  description: string | null;
  author: string | null;
  imageDataUrl: string | null;
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
