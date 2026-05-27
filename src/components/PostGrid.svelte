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

  let cardRefs: (HTMLButtonElement | null)[] = $state([]);

  function rovingTabindex(postId: number, index: number) {
    if (selectedId === null) return index === 0 ? 0 : -1;
    return postId === selectedId ? 0 : -1;
  }

  function focusCard(index: number) {
    const ref = cardRefs[index];
    if (ref) ref.focus({ preventScroll: false });
  }

  function moveBy(delta: number, currentIndex: number) {
    if (posts.length === 0) return;
    const next = Math.min(posts.length - 1, Math.max(0, currentIndex + delta));
    if (next === currentIndex) return;
    onSelect(posts[next].id);
    queueMicrotask(() => focusCard(next));
  }

  function moveVertical(direction: 1 | -1, currentIndex: number) {
    if (posts.length === 0) return;
    const currentEl = cardRefs[currentIndex];
    if (!currentEl) return;
    const currentRect = currentEl.getBoundingClientRect();
    const centerX = currentRect.left + currentRect.width / 2;
    const currentRow = Math.round(currentRect.top);

    let bestIndex = -1;
    let bestDx = Infinity;
    let bestDyRow = direction > 0 ? Infinity : -Infinity;

    for (let i = 0; i < cardRefs.length; i++) {
      const el = cardRefs[i];
      if (!el || i === currentIndex) continue;
      const rect = el.getBoundingClientRect();
      const row = Math.round(rect.top);
      const isBelow = row > currentRow + 1;
      const isAbove = row < currentRow - 1;
      if (direction > 0 && !isBelow) continue;
      if (direction < 0 && !isAbove) continue;

      const rowQualifies =
        direction > 0 ? row <= bestDyRow : row >= bestDyRow;
      const sameRow = row === bestDyRow;
      const dx = Math.abs(rect.left + rect.width / 2 - centerX);
      if (rowQualifies && (!sameRow || dx < bestDx)) {
        bestDyRow = row;
        bestDx = dx;
        bestIndex = i;
      }
    }

    if (bestIndex >= 0) {
      onSelect(posts[bestIndex].id);
      queueMicrotask(() => focusCard(bestIndex));
    }
  }

  function onCardKeydown(event: KeyboardEvent, index: number, post: Post) {
    if (event.key === "ArrowLeft") {
      event.preventDefault();
      event.stopPropagation();
      moveBy(-1, index);
    } else if (event.key === "ArrowRight") {
      event.preventDefault();
      event.stopPropagation();
      moveBy(1, index);
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      event.stopPropagation();
      moveVertical(-1, index);
    } else if (event.key === "ArrowDown") {
      event.preventDefault();
      event.stopPropagation();
      moveVertical(1, index);
    } else if (event.key === "Home") {
      event.preventDefault();
      event.stopPropagation();
      if (posts.length > 0) {
        onSelect(posts[0].id);
        queueMicrotask(() => focusCard(0));
      }
    } else if (event.key === "End") {
      event.preventDefault();
      event.stopPropagation();
      if (posts.length > 0) {
        const last = posts.length - 1;
        onSelect(posts[last].id);
        queueMicrotask(() => focusCard(last));
      }
    } else if (event.key === "Enter") {
      event.preventDefault();
      onOpenOriginal(post);
    }
  }
</script>

<div class="order-2 overflow-auto p-3" data-grid-scroll>
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
    <div
      class="grid gap-2"
      style="grid-template-columns: repeat(auto-fill, minmax(var(--clowder-tile-min, 176px), 1fr));"
      role="grid"
    >
      {#each posts as post, index (post.id)}
        {@const isSelected = selectedId === post.id}
        <button
          bind:this={cardRefs[index]}
          class="group flex min-h-0 flex-col overflow-hidden rounded-[3px] border bg-room-panel text-left transition-colors duration-150 {isSelected
            ? 'border-room-accent bg-room-panel-hi'
            : 'border-room-line hover:border-room-line-strong'}"
          type="button"
          role="gridcell"
          tabindex={rovingTabindex(post.id, index)}
          onclick={() => onSelect(post.id)}
          ondblclick={() => onOpenOriginal(post)}
          onkeydown={(event) => onCardKeydown(event, index, post)}
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
