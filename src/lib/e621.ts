import { invoke } from "@tauri-apps/api/core";
import type { AccountResponse, Post, SearchResponse, TagSuggestion } from "./types";

export const POSTS_PER_PAGE = 64;

export function searchPosts(tags: string, page: number) {
  return invoke<SearchResponse>("search_posts", { tags, page });
}

export function fetchPreview(url: string) {
  return invoke<{ data_url: string }>("fetch_preview", { url });
}

export function mediaUrl(url: string) {
  return invoke<string>("media_url", { url });
}

export function fetchTagSuggestions(term: string) {
  return invoke<TagSuggestion[]>("autocomplete_tags", { term });
}

export function getAccount() {
  return invoke<AccountResponse>("get_account");
}

export function signIn(username: string, apiKey: string) {
  return invoke<AccountResponse>("sign_in", { username, apiKey });
}

export function signOutAccount() {
  return invoke("sign_out");
}

export function favoritePost(postId: number) {
  return invoke("favorite_post", { postId });
}

export function unfavoritePost(postId: number) {
  return invoke("unfavorite_post", { postId });
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
