import { favoritePost, unfavoritePost, updatePostTags } from "./e621";
import { errMsg } from "./errors";
import type { SearchStore } from "./search-store.svelte";
import type { Site } from "./site";
import type { ViewerStore } from "./viewer-store.svelte";
import type { Post } from "./types";

export class PostActionsStore {
  readonly site: Site;
  private readonly search: SearchStore;
  private readonly viewer: ViewerStore;
  favoritePending = $state<Record<number, boolean>>({});

  constructor(site: Site, search: SearchStore, viewer: ViewerStore) {
    this.site = site;
    this.search = search;
    this.viewer = viewer;
  }

  isFavoritePending(id: number): boolean {
    return !!this.favoritePending[id];
  }

  async toggleFavorite(post: Post): Promise<string | null> {
    if (this.favoritePending[post.id]) return null;

    const wasFavorited = post.is_favorited === true;
    this.favoritePending[post.id] = true;

    try {
      if (wasFavorited) {
        await unfavoritePost(this.site, post.id);
      } else {
        await favoritePost(this.site, post.id);
      }
      const patch: Partial<Post> = {
        is_favorited: !wasFavorited,
        fav_count: Math.max(0, (post.fav_count ?? 0) + (wasFavorited ? -1 : 1)),
      };
      this.search.updatePost(post.id, patch);
      this.viewer.updateViewerPost(post.id, patch);
      return null;
    } catch (error) {
      return errMsg(error);
    } finally {
      delete this.favoritePending[post.id];
    }
  }

  async updateTags(post: Post, tagStringDiff: string, editReason: string): Promise<void> {
    const updated = await updatePostTags(this.site, post.id, tagStringDiff, editReason);
    this.search.replacePost(updated);
    this.viewer.replaceViewerPost(updated);
  }
}
