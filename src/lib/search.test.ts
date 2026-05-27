import { describe, expect, it } from "vitest";
import {
  applySuggestionToQuery,
  categoryLabel,
  compact,
  currentToken,
  dimsLabel,
  localMetatagSuggestions,
  looksLikeMetatag,
  postLabel,
  queryWithSort,
  scoreTotal,
  sortModeFromQuery,
  stripSort,
  tagAutocompleteTarget,
} from "./search";
import type { Post } from "./types";

describe("currentToken", () => {
  it("returns the trailing token after the last whitespace", () => {
    const t = currentToken("foo bar baz");
    expect(t.raw).toBe("baz");
    expect(t.start).toBe(8);
    expect(t.end).toBe(11);
    expect(t.search).toBe("baz");
  });

  it("strips a leading negation and normalises spaces to underscores in `search`", () => {
    const t = currentToken("alpha -beta gamma");
    expect(t.raw).toBe("gamma");
    const t2 = currentToken("-hello world");
    expect(t2.raw).toBe("world");
    expect(t2.search).toBe("world");
  });

  it("handles empty input", () => {
    const t = currentToken("");
    expect(t.raw).toBe("");
    expect(t.search).toBe("");
    expect(t.start).toBe(0);
    expect(t.end).toBe(0);
  });
});

describe("looksLikeMetatag", () => {
  it("recognises prefixes of known metatag keys", () => {
    expect(looksLikeMetatag("rat")).toBe(true);
    expect(looksLikeMetatag("ord")).toBe(true);
    expect(looksLikeMetatag("favc")).toBe(true);
    expect(looksLikeMetatag("foo")).toBe(false);
  });
});

describe("tagAutocompleteTarget", () => {
  it("returns unqualified target for bare words", () => {
    const t = tagAutocompleteTarget("kit");
    expect(t).toEqual({ term: "kit", category: null, insertPrefix: "", qualified: false });
  });

  it("strips the leading '-' (negation)", () => {
    const t = tagAutocompleteTarget("-kit");
    expect(t?.term).toBe("kit");
    expect(t?.qualified).toBe(false);
  });

  it("parses a known category prefix", () => {
    const t = tagAutocompleteTarget("artist:al");
    expect(t).toEqual({ term: "al", category: 1, insertPrefix: "artist:", qualified: true });
  });

  it("returns null for unknown prefixes", () => {
    expect(tagAutocompleteTarget("bogus:al")).toBeNull();
  });
});

describe("localMetatagSuggestions", () => {
  it("returns suggestions whose insert or label starts with the term", () => {
    const r = localMetatagSuggestions("rating:s");
    expect(r.map((s) => s.insert)).toContain("rating:s");
  });

  it("returns an empty list for negated inputs", () => {
    expect(localMetatagSuggestions("-rating:s")).toEqual([]);
  });

  it("returns an empty list for empty input", () => {
    expect(localMetatagSuggestions("")).toEqual([]);
  });
});

describe("applySuggestionToQuery", () => {
  it("replaces the trailing token with the suggestion + trailing space", () => {
    const out = applySuggestionToQuery("alpha be", {
      id: 1,
      name: "beta",
      post_count: 0,
      category: 0,
    });
    expect(out).toBe("alpha beta ");
  });

  it("preserves a leading negation on the active token", () => {
    const out = applySuggestionToQuery("alpha -be", {
      id: 1,
      name: "beta",
      post_count: 0,
      category: 0,
    });
    expect(out).toBe("alpha -beta ");
  });

  it("uses suggestion.insert when provided", () => {
    const out = applySuggestionToQuery("ar", {
      id: 1,
      name: "alice",
      post_count: 0,
      category: 1,
      insert: "artist:alice",
    });
    expect(out).toBe("artist:alice ");
  });
});

describe("queryWithSort / stripSort / sortModeFromQuery", () => {
  it("appends order:score when popular", () => {
    expect(queryWithSort("fox", "popular")).toBe("fox order:score");
  });

  it("strips any existing order: token before applying the sort", () => {
    expect(queryWithSort("fox order:rank", "popular")).toBe("fox order:score");
    expect(queryWithSort("fox order:rank", "latest")).toBe("fox");
  });

  it("stripSort removes order tokens, regardless of position", () => {
    expect(stripSort("a order:rank b")).toBe("a b");
    expect(stripSort("order:score")).toBe("");
  });

  it("sortModeFromQuery detects popular by order keywords", () => {
    expect(sortModeFromQuery("a order:score b")).toBe("popular");
    expect(sortModeFromQuery("a order:favcount b")).toBe("popular");
    expect(sortModeFromQuery("a order:rank b")).toBe("latest");
    expect(sortModeFromQuery("a b")).toBe("latest");
  });
});

describe("compact / postLabel / dimsLabel / scoreTotal", () => {
  it("compact formats large numbers", () => {
    expect(compact(0)).toBe("0");
    expect(compact(null)).toBe("0");
    expect(compact(undefined)).toBe("0");
    expect(compact(999)).toBe("999");
    expect(compact(12_500)).toBe("12.5k");
    expect(compact(150_000)).toBe("150k");
  });

  it("postLabel handles 0/1/2/many artists", () => {
    expect(postLabel({ id: 1, file: { ext: "png" } })).toBe("unknown");
    expect(postLabel({ id: 1, file: { ext: "png" }, tags: { artist: [] } })).toBe("unknown");
    expect(postLabel({ id: 1, file: { ext: "png" }, tags: { artist: ["a"] } })).toBe("a");
    expect(postLabel({ id: 1, file: { ext: "png" }, tags: { artist: ["a", "b"] } })).toBe("a + b");
    expect(postLabel({ id: 1, file: { ext: "png" }, tags: { artist: ["a", "b", "c"] } })).toBe(
      "a (+2)",
    );
  });

  it("dimsLabel formats both dimensions or returns '-'", () => {
    expect(dimsLabel({ id: 1, file: { ext: "png", width: 800, height: 600 } })).toBe("800 × 600");
    expect(dimsLabel({ id: 1, file: { ext: "png" } })).toBe("-");
  });

  it("scoreTotal handles missing score", () => {
    expect(scoreTotal({ id: 1, file: { ext: "png" }, score: { total: 7 } })).toBe(7);
    expect(scoreTotal({ id: 1, file: { ext: "png" } })).toBe(0);
  });
});

describe("categoryLabel", () => {
  it("maps known categories", () => {
    expect(categoryLabel(1)).toBe("artist");
    expect(categoryLabel(4)).toBe("character");
    expect(categoryLabel(7)).toBe("meta");
  });

  it("falls back to 'general' for unknown categories", () => {
    expect(categoryLabel(0)).toBe("general");
    expect(categoryLabel(99)).toBe("general");
  });
});
