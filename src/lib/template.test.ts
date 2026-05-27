import { describe, expect, it } from "vitest";
import { applyFilenameTemplate } from "./template";
import type { Post } from "./types";

const samplePost: Post = {
  id: 123,
  file: { ext: "png" },
  tags: { artist: ["alice"] },
  score: { up: 12, down: 3, total: 9 },
  fav_count: 42,
};

describe("applyFilenameTemplate", () => {
  it("substitutes known tokens", () => {
    const out = applyFilenameTemplate("{artist}_{id}.{ext}", samplePost);
    expect(out).toBe("alice_123.png");
  });

  it("falls back to 'unknown_artist' when no artist tags", () => {
    const post: Post = { ...samplePost, tags: { artist: [] } };
    expect(applyFilenameTemplate("{artist}_{id}.{ext}", post)).toBe("unknown_artist_123.png");
  });

  it("defaults extension to jpg when missing", () => {
    const post: Post = { ...samplePost, file: { ext: "" } };
    expect(applyFilenameTemplate("x.{ext}", post)).toBe("x.jpg");
  });

  it("renders numeric fields", () => {
    expect(applyFilenameTemplate("{id}_{score}_{fav_count}", samplePost)).toBe("123_9_42");
  });

  it("leaves unknown tokens untouched", () => {
    expect(applyFilenameTemplate("{artist}-{nope}.{ext}", samplePost)).toBe("alice-{nope}.png");
  });

  it("treats missing score/fav_count as 0", () => {
    const post: Post = { ...samplePost, score: undefined, fav_count: undefined };
    expect(applyFilenameTemplate("{score}_{fav_count}", post)).toBe("0_0");
  });
});
