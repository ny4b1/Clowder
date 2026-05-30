import { mediaUrl, searchPosts, thumbnailUrl } from "./e621";
import { errMsg } from "./errors";
import { queryWithSort, sortModeFromQuery } from "./search";
import { basePresetsBySite, type Site } from "./site";
import { toastStore } from "./toast-store.svelte";
import type { Post, Preset, SortMode } from "./types";

export class SearchStore {
  readonly site: Site;
  readonly basePresets: Preset[];

  query = $state("");
  status = $state("idle");
  posts = $state<Post[]>([]);
  selectedId = $state<number | null>(null);
  loading = $state(false);
  hasSearched = $state(false);
  previews = $state<Record<number, string>>({});
  failedPreviews = $state<Record<number, boolean>>({});
  activePreset = $state<string | null>(null);
  sortMode = $state<SortMode>("latest");
  page = $state(1);
  hasNextPage = $state(false);
  private searchGeneration = 0;

  constructor(site: Site) {
    this.site = site;
    this.basePresets = basePresetsBySite[site];
    this.query = this.basePresets[0].value;
    this.activePreset = this.basePresets[0].value;
  }

  setQuery(value: string) {
    this.query = value;
    this.activePreset = null;
  }

  applySort(value: SortMode) {
    this.sortMode = value;
    this.activePreset = null;
  }

  applyPreset(preset: Preset) {
    this.query = preset.value;
    this.activePreset = preset.value;
    this.sortMode = sortModeFromQuery(preset.value);
  }

  setQueryForTag(tag: string) {
    this.query = tag;
    this.activePreset = null;
  }

  selectId(id: number | null) {
    this.selectedId = id;
  }

  postById(id: number): Post | undefined {
    return this.posts.find((p) => p.id === id);
  }

  updatePost(id: number, patch: Partial<Post>) {
    this.posts = this.posts.map((p) => (p.id === id ? { ...p, ...patch } : p));
  }

  replacePost(post: Post) {
    this.posts = this.posts.map((p) => (p.id === post.id ? post : p));
  }

  moveSelection(delta: number): Post | null {
    if (this.posts.length === 0) return null;
    const currentIndex =
      this.selectedId === null ? -1 : this.posts.findIndex((p) => p.id === this.selectedId);
    const fallbackIndex = delta > 0 ? 0 : this.posts.length - 1;
    const nextIndex =
      currentIndex === -1
        ? fallbackIndex
        : Math.min(this.posts.length - 1, Math.max(0, currentIndex + delta));
    const nextPost = this.posts[nextIndex];
    this.selectedId = nextPost?.id ?? null;
    return nextPost ?? null;
  }

  markPreviewFailed(postId: number) {
    delete this.previews[postId];
    this.failedPreviews[postId] = true;
  }

  async search(targetPage: number, limit: number) {
    const generation = ++this.searchGeneration;
    const trimmed = this.query.trim();
    const hasExplicitOrder = /\border:[a-z_]+/i.test(trimmed);
    if (hasExplicitOrder && !this.activePreset) {
      this.sortMode = sortModeFromQuery(trimmed);
    }
    const tags =
      this.activePreset || hasExplicitOrder ? trimmed : queryWithSort(this.query, this.sortMode);
    this.page = Math.max(1, targetPage);
    this.loading = true;
    this.hasSearched = true;
    this.status = `searching ${tags || "all"}`;
    this.posts = [];
    this.previews = {};
    this.failedPreviews = {};
    this.selectedId = null;
    this.hasNextPage = false;

    try {
      const result = await searchPosts(this.site, tags, this.page, limit);
      if (generation !== this.searchGeneration) return;
      this.posts = result.posts;
      this.hasNextPage = this.posts.length >= limit;
      this.status = `${this.posts.length} post${this.posts.length === 1 ? "" : "s"}`;
      for (const post of this.posts) {
        void this.loadPreview(post, generation);
      }
    } catch (error) {
      if (generation !== this.searchGeneration) return;
      const message = `search failed: ${errMsg(error)}`;
      this.status = message;
      toastStore.error(message);
    } finally {
      if (generation === this.searchGeneration) {
        this.loading = false;
      }
    }
  }

  private async loadPreview(post: Post, generation: number) {
    if (generation !== this.searchGeneration) return;
    const url = thumbnailUrl(post);
    if (!url) {
      if (generation === this.searchGeneration) {
        this.markPreviewFailed(post.id);
      }
      return;
    }
    try {
      const proxied = await mediaUrl(url);
      if (generation === this.searchGeneration) {
        this.previews[post.id] = proxied;
      }
    } catch {
      if (generation === this.searchGeneration) {
        this.markPreviewFailed(post.id);
      }
    }
  }
}
