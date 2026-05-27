import type { Post } from "./types";

export const DEFAULT_FILENAME_TEMPLATE = "{artist}_{id}.{ext}";

export const FILENAME_TOKENS = [
  { token: "{artist}", description: "first artist tag (or 'unknown_artist')" },
  { token: "{id}", description: "post id" },
  { token: "{ext}", description: "file extension (jpg / png / webm / ...)" },
  { token: "{score}", description: "total score" },
  { token: "{fav_count}", description: "favorite count" },
] as const;

export function applyFilenameTemplate(template: string, post: Post): string {
  const tokens: Record<string, string> = {
    artist: post.tags?.artist?.[0] || "unknown_artist",
    id: String(post.id),
    ext: post.file?.ext || "jpg",
    score: String(post.score?.total ?? 0),
    fav_count: String(post.fav_count ?? 0),
  };
  return template.replace(/\{(\w+)\}/g, (raw, key: string) => tokens[key] ?? raw);
}
