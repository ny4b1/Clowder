import { favoritePost, unfavoritePost, updatePostTags } from "./e621";
import { searchStore } from "./search-store.svelte";
import { viewerStore } from "./viewer-store.svelte";
import type { Post } from "./types";

class PostActionsStore {
  favoritePending = $state<Record<number, boolean>>({});

  isFavoritePending(id: number): boolean {
    return !!this.favoritePending[id];
  }

  async toggleFavorite(post: Post): Promise<string | null> {
    if (this.favoritePending[post.id]) return null;

    const wasFavorited = post.is_favorited === true;
    this.favoritePending[post.id] = true;
    this.favoritePending = { ...this.favoritePending };

    try {
      if (wasFavorited) {
        await unfavoritePost(post.id);
      } else {
        await favoritePost(post.id);
      }
      const patch: Partial<Post> = {
        is_favorited: !wasFavorited,
        fav_count: Math.max(0, (post.fav_count ?? 0) + (wasFavorited ? -1 : 1)),
      };
      searchStore.updatePost(post.id, patch);
      viewerStore.updateViewerPost(post.id, patch);
      return null;
    } catch (error) {
      return String(error);
    } finally {
      delete this.favoritePending[post.id];
      this.favoritePending = { ...this.favoritePending };
    }
  }

  async updateTags(post: Post, tagStringDiff: string, editReason: string): Promise<void> {
    const updated = await updatePostTags(post.id, tagStringDiff, editReason);
    searchStore.replacePost(updated);
    viewerStore.replaceViewerPost(updated);
  }
}

export const postActionsStore = new PostActionsStore();
