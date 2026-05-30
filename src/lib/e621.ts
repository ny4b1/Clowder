import { invoke } from "@tauri-apps/api/core";
import type { Site } from "./site";
import type { AccountResponse, Comment, Post, SearchResponse, TagSuggestion } from "./types";

export function searchPosts(site: Site, tags: string, page: number, limit: number) {
  return invoke<SearchResponse>("search_posts", { site, tags, page, limit });
}

export function mediaUrl(url: string) {
  return invoke<string>("media_url", { url });
}

export function fetchTagSuggestions(site: Site, term: string, category: number | null = null) {
  return invoke<TagSuggestion[]>("autocomplete_tags", { site, term, category });
}

export function getAccount(site: Site) {
  return invoke<AccountResponse>("get_account", { site });
}

export function signIn(site: Site, username: string, apiKey: string) {
  return invoke<AccountResponse>("sign_in", { site, username, apiKey });
}

export function signOutAccount(site: Site) {
  return invoke("sign_out", { site });
}

export function favoritePost(site: Site, postId: number) {
  return invoke("favorite_post", { site, postId });
}

export function unfavoritePost(site: Site, postId: number) {
  return invoke("unfavorite_post", { site, postId });
}

export function fetchComments(site: Site, postId: number, limit = 40) {
  return invoke<Comment[]>("fetch_comments", { site, postId, limit });
}

export function createComment(site: Site, postId: number, body: string) {
  return invoke<Comment>("create_comment", { site, postId, body });
}

export function hideComment(site: Site, commentId: number) {
  return invoke<Comment>("hide_comment", { site, commentId });
}

export function updatePostTags(
  site: Site,
  postId: number,
  tagStringDiff: string,
  editReason: string,
) {
  return invoke<Post>("update_post_tags", { site, postId, tagStringDiff, editReason });
}

export function downloadFile(url: string, filename: string) {
  return invoke<string>("download_file", { url, filename });
}

export function thumbnailUrl(post: Post) {
  return firstImageUrl(post.preview?.url, post.sample?.url, post.file?.url);
}

export function firstImageUrl(...urls: Array<string | null | undefined>) {
  return urls.find((url) => url && isImageUrl(url)) ?? "";
}

export function isImageUrl(url: string) {
  try {
    const path = new URL(url).pathname.toLowerCase();
    return /\.(jpe?g|png|gif|webp)$/.test(path);
  } catch {
    return /\.(jpe?g|png|gif|webp)(?:[?#].*)?$/i.test(url);
  }
}

export function originalUrl(post: Post) {
  return post.file?.url || "";
}

export function isVideoPost(post: Post) {
  return ["webm", "mp4"].includes((post.file?.ext || "").toLowerCase());
}
