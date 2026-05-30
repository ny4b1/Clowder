<script lang="ts">
  import HeartIcon from "./icons/HeartIcon.svelte";
  import TagGroup from "./TagGroup.svelte";
  import type { Post } from "../lib/types";
  import { dimsLabel, postLabel, tagGroups } from "../lib/search";
  import type { Site } from "../lib/site";

  type Props = {
    selectedPost: Post | null;
    site: Site;
    username: string | null;
    favoritePending: Record<number, boolean>;
    onOpenOriginal: (post: Post) => void;
    onToggleFavorite: (post: Post) => void;
  };

  let { selectedPost, site, username, favoritePending, onOpenOriginal, onToggleFavorite }: Props =
    $props();
</script>

<aside class="order-1 flex min-h-0 flex-col border-r border-room-line bg-room-panel/40">
  {#if selectedPost}
    <div class="overflow-auto">
      <section class="border-b border-room-line px-5 py-4">
        <div class="font-mono text-[10px] uppercase tracking-[0.22em] text-room-text-low">
          post
        </div>
        <div class="mt-1.5 font-mono text-[15px] tabular-nums text-room-text">
          #{selectedPost.id}
        </div>
        <div class="mt-0.5 truncate text-[12.5px] text-room-text-mid">
          {postLabel(selectedPost, site)}
        </div>
        <div class="mt-3 flex items-center gap-4 font-mono text-[11px] tabular-nums">
          <div class="flex items-center gap-1.5 text-room-text">
            <svg
              class="size-3.5 text-room-text-mid"
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
            <span>{(selectedPost.score?.total ?? 0).toLocaleString()}</span>
            <span class="text-room-text-low">
              ({selectedPost.score?.up ?? 0}/{selectedPost.score?.down ?? 0})
            </span>
          </div>
          <div
            class="flex items-center gap-1.5 {selectedPost.is_favorited
              ? 'text-room-fav'
              : 'text-room-text'}"
          >
            <HeartIcon filled={!!selectedPost.is_favorited} class="size-3.5" />
            <span>{(selectedPost.fav_count ?? 0).toLocaleString()}</span>
          </div>
        </div>
      </section>

      <section class="border-b border-room-line px-5 py-4">
        <div class="mb-2.5 font-mono text-[10px] uppercase tracking-[0.22em] text-room-text-low">
          file
        </div>
        <dl class="grid grid-cols-[60px_minmax(0,1fr)] gap-y-1.5 text-[11.5px]">
          <dt class="text-room-text-mid">format</dt>
          <dd class="font-mono text-room-text">
            {(selectedPost.file?.ext || "-").toUpperCase()}
          </dd>
          <dt class="text-room-text-mid">dims</dt>
          <dd class="font-mono tabular-nums text-room-text">{dimsLabel(selectedPost)}</dd>
        </dl>
        <div class="mt-3 flex flex-wrap gap-1.5">
          {#if selectedPost.file?.url}
            <button
              type="button"
              onclick={() => onOpenOriginal(selectedPost)}
              class="inline-flex h-7 items-center gap-1.5 rounded-[3px] border border-room-line-strong bg-room-panel px-2.5 font-mono text-[10.5px] uppercase tracking-[0.18em] text-room-text-mid no-underline transition-colors duration-150 hover:border-room-accent hover:text-room-accent"
            >
              open original
              <svg
                class="size-3"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
                aria-hidden="true"
              >
                <path d="M7 17 17 7" />
                <path d="M7 7h10v10" />
              </svg>
            </button>
          {/if}
          <button
            type="button"
            onclick={() => onToggleFavorite(selectedPost)}
            disabled={!!favoritePending[selectedPost.id]}
            class="inline-flex h-7 items-center gap-1.5 rounded-[3px] border bg-room-panel px-2.5 font-mono text-[10.5px] uppercase tracking-[0.18em] transition-colors duration-150 disabled:opacity-50 {selectedPost.is_favorited
              ? 'border-room-fav text-room-fav hover:bg-room-fav/10'
              : 'border-room-line-strong text-room-text-mid hover:border-room-fav hover:text-room-fav'}"
          >
            <HeartIcon filled={!!selectedPost.is_favorited} />
            {selectedPost.is_favorited ? "favorited" : "favorite"}
          </button>
        </div>
        {#if !username}
          <p class="mt-2 font-mono text-[10px] text-room-text-low">sign in to favorite</p>
        {/if}
      </section>

      {#each tagGroups(selectedPost, site) as [group, tags] (group)}
        <TagGroup {group} {tags} maxVisible={32} />
      {/each}
    </div>
  {:else}
    <div class="grid h-full place-items-center px-6">
      <div class="text-center">
        <div class="font-mono text-[10px] uppercase tracking-[0.25em] text-room-text-low">
          no selection
        </div>
        <p class="mt-2 text-[12px] text-room-text-mid">Select a post to inspect.</p>
      </div>
    </div>
  {/if}
</aside>
