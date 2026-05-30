import type { PostTags, Preset } from "./types";

export type Site = "e621" | "e6ai";

export const SITES: readonly Site[] = ["e621", "e6ai"];

export const DEFAULT_SITE: Site = "e621";

export const siteLabels: Record<Site, string> = {
  e621: "e621",
  e6ai: "e6ai",
};

export const basePresetsBySite: Record<Site, Preset[]> = {
  e621: [
    { label: "Hot", value: "order:rank" },
    { label: "Popular Today", value: "date:day order:score" },
  ],
  e6ai: [
    { label: "Hot", value: "order:rank" },
    { label: "Popular Today", value: "date:day order:score" },
  ],
};

export type TagGroupDef = {
  key: keyof PostTags;
  label: string;
  category: number;
};

export const tagTaxonomy: Record<Site, TagGroupDef[]> = {
  e621: [
    { key: "artist", label: "artist", category: 1 },
    { key: "copyright", label: "copyright", category: 3 },
    { key: "character", label: "character", category: 4 },
    { key: "species", label: "species", category: 5 },
    { key: "general", label: "general", category: 0 },
    { key: "meta", label: "meta", category: 7 },
    { key: "lore", label: "lore", category: 8 },
  ],
  e6ai: [
    { key: "director", label: "director", category: 1 },
    { key: "contributor", label: "contributor", category: 2 },
    { key: "franchise", label: "franchise", category: 3 },
    { key: "character", label: "character", category: 4 },
    { key: "species", label: "species", category: 5 },
    { key: "general", label: "general", category: 0 },
    { key: "meta", label: "meta", category: 7 },
    { key: "lore", label: "lore", category: 8 },
  ],
};

export function primaryCreatorKey(site: Site): keyof PostTags {
  return tagTaxonomy[site][0].key;
}
