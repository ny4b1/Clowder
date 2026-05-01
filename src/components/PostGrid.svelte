<script lang="ts">
  import type { Post } from "../lib/types";
  import { compact, postLabel, scoreTotal } from "../lib/search";

  type Props = {
    posts: Post[];
    loading: boolean;
    hasSearched: boolean;
    selectedId: number | null;
    previews: Record<number, string>;
    failedPreviews: Record<number, boolean>;
    onSelect: (id: number) => void;
    onOpenOriginal: (post: Post) => void;
    onPreviewError: (postId: number) => void;
  };

  let {
    posts,
    loading,
    hasSearched,
    selectedId,
    previews,
    failedPreviews,
    onSelect,
    onOpenOriginal,
    onPreviewError,
  }: Props = $props();
</script>

<div class="order-2 overflow-auto p-3">
  {#if posts.length === 0}
    <div class="grid h-full place-items-center px-6">
      <div class="max-w-md text-center">
        {#if loading}
          <span
            class="inline-block size-4 animate-spin rounded-full border border-room-line-strong border-t-room-accent"
            aria-hidden="true"
          ></span>
          <div class="mt-3 font-mono text-[10px] uppercase tracking-[0.25em] text-room-text-low">
            searching
          </div>
        {:else if hasSearched}
          <div class="mb-3 font-mono text-[10px] uppercase tracking-[0.25em] text-room-text-low">
            no results
          </div>
          <p class="text-[13px] text-room-text-mid">No posts match this query.</p>
        {:else}
          <div class="mb-3 font-mono text-[10px] uppercase tracking-[0.25em] text-room-text-low">
            empty workspace
          </div>
          <h2 class="text-[20px] font-light leading-tight tracking-tight text-room-text">
            Search e621 posts
          </h2>
          <p class="mt-2 text-[13px] leading-relaxed text-room-text-mid">
            Type tags above. Use arrow keys for autocomplete, Tab or Enter to insert.
          </p>
        {/if}
      </div>
    </div>
  {:else}
    <div class="grid grid-cols-[repeat(auto-fill,minmax(176px,1fr))] gap-2">
      {#each posts as post (post.id)}
        {@const isSelected = selectedId === post.id}
        <button
          class="group flex min-h-0 flex-col overflow-hidden rounded-[3px] border bg-room-panel text-left transition-colors duration-150 {isSelected
            ? 'border-room-accent bg-room-panel-hi'
            : 'border-room-line hover:border-room-line-strong'}"
          type="button"
          onclick={() => onSelect(post.id)}
          ondblclick={() => onOpenOriginal(post)}
        >
          <div class="relative aspect-square min-h-0 overflow-hidden bg-room-floor">
            {#if previews[post.id]}
              <img
                class="absolute inset-0 h-full w-full object-cover"
                src={previews[post.id]}
                alt={postLabel(post)}
                draggable="false"
                onerror={() => onPreviewError(post.id)}
              />
            {:else if failedPreviews[post.id]}
              <span class="absolute inset-0 grid place-items-center font-mono text-[9.5px] uppercase tracking-[0.2em] text-room-text-low">
                no preview
              </span>
            {:else}
              <span
                class="absolute left-1/2 top-1/2 size-3 -translate-x-1/2 -translate-y-1/2 animate-spin rounded-full border border-room-line-strong border-t-room-text-mid"
                aria-hidden="true"
              ></span>
            {/if}
          </div>
          <div class="h-16 overflow-hidden border-t border-room-line px-2.5 py-1.5">
            <div class="flex items-baseline gap-1.5">
              <span class="font-mono text-[10.5px] tabular-nums text-room-text-low">
                #{post.id}
              </span>
              <span class="truncate text-[11.5px] text-room-text">
                {postLabel(post)}
              </span>
            </div>
            <div class="mt-0.5 flex items-center gap-2.5 font-mono text-[10px] tabular-nums">
              <span class="inline-flex items-center gap-1 text-room-text-mid">
                <svg
                  class="size-3 shrink-0"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  aria-hidden="true"
                >
                  <path d="m6 15 6-6 6 6" />
                </svg>
                {compact(scoreTotal(post))}
              </span>
              <span
                class="inline-flex items-center gap-1 {post.is_favorited
                  ? 'text-room-fav'
                  : 'text-room-text-mid'}"
              >
                <svg
                  class="size-3 shrink-0"
                  viewBox="0 0 24 24"
                  fill={post.is_favorited ? "currentColor" : "none"}
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  aria-hidden="true"
                >
                  <path
                    d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"
                  />
                </svg>
                {compact(post.fav_count ?? 0)}
              </span>
            </div>
          </div>
        </button>
      {/each}
    </div>
  {/if}
</div>
