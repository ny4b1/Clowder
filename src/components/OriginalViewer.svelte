<script lang="ts">
  import type { OriginalViewer as OriginalViewerState, Post } from "../lib/types";
  import { isVideoPost } from "../lib/e621";
  import { dimsLabel, postLabel, tagGroups } from "../lib/search";

  type Props = {
    viewer: OriginalViewerState;
    username: string | null;
    favoritePending: boolean;
    downloadPending: boolean;
    downloadStatus: string;
    onClose: () => void;
    onSearchTag: (tag: string) => void;
    onToggleFavorite: (post: Post) => void;
    onDownload: (post: Post) => void;
  };

  let {
    viewer,
    username,
    favoritePending,
    downloadPending,
    downloadStatus,
    onClose,
    onSearchTag,
    onToggleFavorite,
    onDownload,
  }: Props = $props();
</script>

<div class="fixed inset-0 z-40 grid grid-rows-[42px_minmax(0,1fr)] bg-room-floor">
  <div class="flex items-center justify-between border-b border-room-line bg-room-panel px-4">
    <div class="min-w-0">
      <div class="font-mono text-[10px] uppercase tracking-[0.22em] text-room-text-low">
        original
      </div>
      <div class="truncate font-mono text-[12px] tabular-nums text-room-text">
        #{viewer.post.id}
        <span class="text-room-text-low">
          {(viewer.post.file?.ext || "").toUpperCase()} {dimsLabel(viewer.post)}
        </span>
      </div>
    </div>
    <button
      type="button"
      onclick={onClose}
      class="flex size-8 items-center justify-center rounded-[3px] border border-room-line bg-room-panel text-room-text-mid transition-colors duration-150 hover:border-room-line-strong hover:text-room-text"
      aria-label="Close original"
    >
      <svg
        class="size-3.5"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
        aria-hidden="true"
      >
        <path d="M18 6 6 18" />
        <path d="m6 6 12 12" />
      </svg>
    </button>
  </div>

  <div class="grid min-h-0 grid-cols-[300px_minmax(0,1fr)]">
    <aside class="min-h-0 overflow-auto border-r border-room-line bg-room-panel/40">
      <section class="border-b border-room-line px-4 py-3">
        <div class="font-mono text-[10px] uppercase tracking-[0.22em] text-room-text-low">
          post
        </div>
        <div class="mt-1.5 font-mono text-[14px] tabular-nums text-room-text">
          #{viewer.post.id}
        </div>
        <div class="mt-0.5 truncate text-[12px] text-room-text-mid">
          {postLabel(viewer.post)}
        </div>
      </section>

      {#each tagGroups(viewer.post) as [group, tags] (group)}
        {#if tags.length > 0}
          <section class="border-b border-room-line px-4 py-3">
            <div class="mb-2 flex items-baseline justify-between">
              <div class="font-mono text-[10px] uppercase tracking-[0.22em] text-room-text-low">
                {group}
              </div>
              <div class="font-mono text-[10px] tabular-nums text-room-text-low">
                {tags.length}
              </div>
            </div>
            <div class="flex flex-wrap gap-1">
              {#each tags as tag}
                <button
                  type="button"
                  onclick={() => onSearchTag(tag)}
                  class="inline-flex h-6 max-w-full items-center truncate rounded-[2px] border border-room-line bg-room-panel px-2 font-mono text-[10.5px] text-room-text-mid transition-colors duration-150 hover:border-room-accent hover:text-room-accent"
                >
                  {tag}
                </button>
              {/each}
            </div>
          </section>
        {/if}
      {/each}
    </aside>

    <div class="min-h-0 overflow-auto">
      <div class="p-4">
        <div class="grid min-h-full place-items-start justify-center">
          {#if viewer.loading}
            <span
              class="mt-10 size-4 animate-spin rounded-full border border-room-line-strong border-t-room-accent"
              aria-hidden="true"
            ></span>
          {:else if viewer.error}
            <div class="mt-10 max-w-lg text-center font-mono text-[11px] leading-relaxed text-room-fav">
              {viewer.error}
            </div>
          {:else if viewer.dataUrl && isVideoPost(viewer.post)}
            <!-- svelte-ignore a11y_media_has_caption -->
            <video
              class="max-w-full"
              src={viewer.dataUrl}
              controls
              autoplay
              loop
            ></video>
          {:else if viewer.dataUrl}
            <img
              class="max-w-full object-contain"
              src={viewer.dataUrl}
              alt={postLabel(viewer.post)}
              draggable="false"
            />
          {/if}
        </div>
      </div>

      <section class="border-t border-room-line bg-room-panel/25 px-4 py-3">
        <div class="flex flex-wrap items-center gap-2">
          <button
            type="button"
            onclick={() => onToggleFavorite(viewer.post)}
            disabled={favoritePending}
            class="inline-flex h-8 items-center gap-1.5 rounded-[3px] border bg-room-panel px-3 font-mono text-[10.5px] uppercase tracking-[0.18em] transition-colors duration-150 disabled:opacity-50 {viewer.post.is_favorited
              ? 'border-room-fav text-room-fav hover:bg-room-fav/10'
              : 'border-room-line-strong text-room-text-mid hover:border-room-fav hover:text-room-fav'}"
          >
            <svg
              class="size-3"
              viewBox="0 0 24 24"
              fill={viewer.post.is_favorited ? "currentColor" : "none"}
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
            {viewer.post.is_favorited ? "favorited" : "favorite"}
          </button>

          <button
            type="button"
            onclick={() => onDownload(viewer.post)}
            disabled={downloadPending || !viewer.post.file?.url}
            class="inline-flex h-8 items-center gap-1.5 rounded-[3px] border border-room-line-strong bg-room-panel px-3 font-mono text-[10.5px] uppercase tracking-[0.18em] text-room-text-mid transition-colors duration-150 hover:border-room-accent hover:text-room-accent disabled:opacity-50"
          >
            {#if downloadPending}
              <span
                class="size-2.5 animate-spin rounded-full border border-room-line-strong border-t-room-accent"
                aria-hidden="true"
              ></span>
            {/if}
            download
          </button>

          {#if !username}
            <span class="font-mono text-[10px] text-room-text-low">sign in to favorite</span>
          {/if}
          {#if downloadStatus}
            <span class="font-mono text-[10.5px] text-room-text-low">{downloadStatus}</span>
          {/if}
        </div>
      </section>
    </div>
  </div>
</div>
