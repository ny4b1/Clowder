import { describe, expect, it } from "vitest";
import { firstImageUrl, isImageUrl, isVideoPost, originalUrl, thumbnailUrl } from "./e621";
import type { Post } from "./types";

describe("isImageUrl", () => {
  it("accepts common image extensions on absolute URLs", () => {
    expect(isImageUrl("https://e621.net/x.jpg")).toBe(true);
    expect(isImageUrl("https://e621.net/x.JPEG")).toBe(true);
    expect(isImageUrl("https://e621.net/x.png")).toBe(true);
    expect(isImageUrl("https://e621.net/x.gif")).toBe(true);
    expect(isImageUrl("https://e621.net/x.webp")).toBe(true);
  });

  it("rejects video and unknown extensions", () => {
    expect(isImageUrl("https://e621.net/x.mp4")).toBe(false);
    expect(isImageUrl("https://e621.net/x.webm")).toBe(false);
    expect(isImageUrl("https://e621.net/no_ext")).toBe(false);
  });

  it("falls back to regex for non-URL strings, supporting query suffixes", () => {
    expect(isImageUrl("image.png?x=1")).toBe(true);
    expect(isImageUrl("image.webm?x=1")).toBe(false);
  });
});

describe("firstImageUrl", () => {
  it("returns the first image URL in order", () => {
    expect(
      firstImageUrl("https://e621.net/v.webm", "https://e621.net/p.png", "https://e621.net/s.jpg"),
    ).toBe("https://e621.net/p.png");
  });

  it("returns an empty string when no candidate is an image", () => {
    expect(firstImageUrl(null, undefined, "https://e621.net/v.webm")).toBe("");
  });
});

describe("thumbnailUrl", () => {
  it("prefers preview, then sample, then file", () => {
    const post: Post = {
      id: 1,
      file: { ext: "png", url: "https://e621.net/file.png" },
      preview: { url: "https://e621.net/prev.jpg" },
      sample: { url: "https://e621.net/sample.png" },
    };
    expect(thumbnailUrl(post)).toBe("https://e621.net/prev.jpg");
  });

  it("skips a video preview and falls through to the next image", () => {
    const post: Post = {
      id: 1,
      file: { ext: "webm", url: "https://e621.net/file.webm" },
      preview: { url: "https://e621.net/prev.webm" },
      sample: { url: "https://e621.net/sample.jpg" },
    };
    expect(thumbnailUrl(post)).toBe("https://e621.net/sample.jpg");
  });
});

describe("originalUrl", () => {
  it("returns the file URL when present", () => {
    expect(
      originalUrl({ id: 1, file: { ext: "png", url: "https://e621.net/file.png" } }),
    ).toBe("https://e621.net/file.png");
  });

  it("returns an empty string when no file URL", () => {
    expect(originalUrl({ id: 1, file: { ext: "png" } })).toBe("");
  });
});

describe("isVideoPost", () => {
  it.each([
    ["webm", true],
    ["WEBM", true],
    ["mp4", true],
    ["MP4", true],
    ["png", false],
    ["jpg", false],
    ["", false],
  ])("ext=%s -> %s", (ext, expected) => {
    expect(isVideoPost({ id: 1, file: { ext } })).toBe(expected);
  });
});
