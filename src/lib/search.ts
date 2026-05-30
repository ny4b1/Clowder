import { DEFAULT_SITE, type Site, primaryCreatorKey, tagTaxonomy } from "./site";
import type { Post, SortMode, TagSuggestion } from "./types";

export function currentToken(value: string) {
  const end = value.length;
  const start =
    Math.max(value.lastIndexOf(" "), value.lastIndexOf("\n"), value.lastIndexOf("\t")) + 1;
  const raw = value.slice(start, end);
  return {
    start,
    end,
    raw,
    search: raw.replace(/^-/, "").replaceAll(" ", "_"),
  };
}

export function looksLikeMetatag(value: string) {
  return ["rating", "order", "fav", "date", "score", "favcount"].some((key) =>
    key.startsWith(value.toLowerCase()),
  );
}

function tagCategoryPrefixes(site: Site): Record<string, number> {
  const prefixes: Record<string, number> = { invalid: 6 };
  for (const def of tagTaxonomy[site]) {
    prefixes[def.label] = def.category;
  }
  return prefixes;
}

export function tagAutocompleteTarget(raw: string, site: Site = DEFAULT_SITE) {
  const value = raw.replace(/^-/, "");
  const match = /^([a-z_]+):(.*)$/i.exec(value);
  if (!match) {
    return {
      term: value.replaceAll(" ", "_"),
      category: null,
      insertPrefix: "",
      qualified: false,
    };
  }

  const prefix = match[1].toLowerCase();
  const category = tagCategoryPrefixes(site)[prefix];
  if (category === undefined) {
    return null;
  }

  return {
    term: match[2].replaceAll(" ", "_"),
    category,
    insertPrefix: `${prefix}:`,
    qualified: true,
  };
}

export function localMetatagSuggestions(raw: string): TagSuggestion[] {
  const negative = raw.startsWith("-");
  const value = raw.replace(/^-/, "").toLowerCase();
  if (!value || negative) return [];

  const options = [
    ["rating:s", "rating:safe"],
    ["rating:q", "rating:questionable"],
    ["rating:e", "rating:explicit"],
    ["order:id_desc", "order:newest"],
    ["order:score", "order:score"],
    ["order:favcount", "order:favorites"],
    ["order:rank", "order:rank"],
  ] as const;

  return options
    .filter(([insert, label]) => insert.startsWith(value) || label.startsWith(value))
    .map(([insert, label], index) => ({
      id: -index - 1,
      name: label,
      insert,
      post_count: 0,
      category: 7,
    }));
}

export function applySuggestionToQuery(query: string, suggestion: TagSuggestion) {
  const token = currentToken(query);
  const prefix = token.raw.startsWith("-") ? "-" : "";
  return `${query.slice(0, token.start)}${prefix}${suggestion.insert ?? suggestion.name} ${query.slice(token.end)}`;
}

export function categoryLabel(category: number, site: Site = DEFAULT_SITE) {
  if (category === 6) return "invalid";
  const def = tagTaxonomy[site].find((entry) => entry.category === category);
  return def?.label ?? "general";
}

export function queryWithSort(value: string, mode: SortMode) {
  const clean = stripSort(value);
  if (mode === "popular") {
    return [clean, "order:score"].filter(Boolean).join(" ");
  }
  return clean;
}

export function stripSort(value: string) {
  return value
    .trim()
    .split(/\s+/)
    .filter((token) => !/^order:/i.test(token))
    .join(" ");
}

export function sortModeFromQuery(value: string): SortMode {
  return /\border:(score|favcount)\b/i.test(value) ? "popular" : "latest";
}

export function postLabel(post: Post, site: Site = DEFAULT_SITE) {
  const creators = post.tags?.[primaryCreatorKey(site)] ?? [];
  if (creators.length === 0) return "unknown";
  if (creators.length === 1) return creators[0];
  if (creators.length === 2) return `${creators[0]} + ${creators[1]}`;
  return `${creators[0]} (+${creators.length - 1})`;
}

export function dimsLabel(post: Post) {
  const w = post.file?.width;
  const h = post.file?.height;
  return w && h ? `${w} × ${h}` : "-";
}

export function compact(n: number | undefined | null) {
  if (n === undefined || n === null || Number.isNaN(n)) return "0";
  const abs = Math.abs(n);
  if (abs >= 100000) return `${Math.round(n / 1000).toLocaleString()}k`;
  if (abs >= 10000) return `${(n / 1000).toFixed(1)}k`;
  return n.toLocaleString();
}

export function scoreTotal(post: Post) {
  return post.score?.total ?? 0;
}

export function tagGroups(post: Post, site: Site = DEFAULT_SITE) {
  return tagTaxonomy[site].map(
    (def) => [def.label, post.tags?.[def.key] ?? []] as [string, string[]],
  );
}
