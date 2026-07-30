import type { AgentDetails } from "../types";

export function resolveAgentImageSrc(baseImage: string | null): string {
  if (!baseImage) return "/images/placeholder.jpg";
  if (baseImage.startsWith("data:")) return baseImage;
  return `/images/entities/${baseImage}`;
}

const EMPTY_DETAILS: AgentDetails = {
  rank: "",
  attribute: "",
  speciality: "",
  type: [],
};

export function parseAgentDetails(details: string | null): AgentDetails {
  if (!details) return { ...EMPTY_DETAILS };
  try {
    const parsed = JSON.parse(details);
    return {
      rank: parsed.rank ?? "",
      attribute: parsed.attribute ?? "",
      speciality: parsed.speciality ?? "",
      type: Array.isArray(parsed.type) ? parsed.type.filter(Boolean) : [],
    };
  } catch {
    return { ...EMPTY_DETAILS };
  }
}

export function serializeAgentDetails(details: AgentDetails): string {
  return JSON.stringify(details);
}
