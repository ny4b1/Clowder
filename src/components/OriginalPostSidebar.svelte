<script lang="ts">
  import Spinner from "./icons/Spinner.svelte";
  import TagGroup from "./TagGroup.svelte";
  import type { Post } from "../lib/types";
  import { errMsg } from "../lib/errors";
  import { postLabel, tagGroups } from "../lib/search";
  import type { Site } from "../lib/site";

  type Props = {
    post: Post;
    site: Site;
    username: string | null;
    onSearchTag: (tag: string) => void;
    onOpenAccount: () => void;
    onUpdateTags: (post: Post, tagStringDiff: string, editReason: string) => Promise<void>;
  };

  let { post, site, username, onSearchTag, onOpenAccount, onUpdateTags }: Props = $props();

  let tagEditorOpen = $state(false);
  let tagStringDiff = $state("");
  let tagEditReason = $state("");
  let tagEditPending = $state(false);
  let tagEditStatus = $state("");
  let lastPostId = $state(0);

  $effect(() => {
    if (post.id === lastPostId) return;
    lastPostId = post.id;
    tagEditorOpen = false;
    tagStringDiff = "";
    tagEditReason = "";
    tagEditPending = false;
    tagEditStatus = "";
  });

  async function submitTagEdit() {
    const diff = tagStringDiff.trim();
    if (!diff || tagEditPending) return;
    tagEditPending = true;
    tagEditStatus = "saving";
    try {
      await onUpdateTags(post, diff, tagEditReason);
      tagStringDiff = "";
      tagEditReason = "";
      tagEditorOpen = false;
      tagEditStatus = "";
    } catch (error) {
      tagEditStatus = errMsg(error);
    } finally {
      tagEditPending = false;
    }
  }
</script>

<aside class="min-h-0 overflow-auto border-r border-room-line bg-room-panel/40">
  <section class="border-b border-room-line px-4 py-3">
    <div class="font-mono text-[10px] uppercase tracking-[0.22em] text-room-text-low">
      post
    </div>
    <div class="mt-1.5 font-mono text-[14px] tabular-nums text-room-text">
      #{post.id}
    </div>
    <div class="mt-0.5 truncate text-[12px] text-room-text-mid">
      {postLabel(post, site)}
    </div>
    <div class="mt-3">
      {#if username}
        <button
          type="button"
          onclick={() => {
            tagEditorOpen = !tagEditorOpen;
            tagEditStatus = "";
          }}
          class="h-7 rounded-[3px] border border-room-line-strong bg-room-panel px-2.5 font-mono text-[10.5px] uppercase tracking-[0.18em] text-room-text-mid transition-colors duration-150 hover:border-room-accent hover:text-room-accent"
        >
          edit tags
        </button>
      {:else}
        <button
          type="button"
          onclick={onOpenAccount}
          class="h-7 rounded-[3px] border border-room-line bg-room-panel px-2.5 font-mono text-[10.5px] uppercase tracking-[0.18em] text-room-text-low transition-colors duration-150 hover:border-room-line-strong hover:text-room-text-mid"
        >
          sign in to edit
        </button>
      {/if}
    </div>
    {#if tagEditorOpen}
      <form
        class="mt-3 space-y-2"
        onsubmit={(event) => {
          event.preventDefault();
          void submitTagEdit();
        }}
      >
        <label>
          <span class="mb-1 block font-mono text-[10px] uppercase tracking-[0.18em] text-room-text-low">
            tag diff
          </span>
          <textarea
            value={tagStringDiff}
            oninput={(event) => (tagStringDiff = event.currentTarget.value)}
            disabled={tagEditPending}
            class="h-20 w-full resize-none rounded-[3px] border border-room-line bg-room-panel px-2 py-1.5 font-mono text-[11px] leading-relaxed text-room-text outline-none transition-colors duration-150 placeholder:text-room-text-low focus:border-room-accent disabled:opacity-50"
            placeholder="tag_to_add -tag_to_remove"
          ></textarea>
        </label>
        <label>
          <span class="mb-1 block font-mono text-[10px] uppercase tracking-[0.18em] text-room-text-low">
            reason
          </span>
          <input
            value={tagEditReason}
            oninput={(event) => (tagEditReason = event.currentTarget.value)}
            disabled={tagEditPending}
            class="h-7 w-full rounded-[3px] border border-room-line bg-room-panel px-2 font-mono text-[11px] text-room-text outline-none transition-colors duration-150 placeholder:text-room-text-low focus:border-room-accent disabled:opacity-50"
            placeholder="optional"
          />
        </label>
        <div class="flex items-center justify-between gap-2">
          <span
            class="min-w-0 truncate font-mono text-[10px] text-room-text-low {tagEditStatus && tagEditStatus !== 'saving'
              ? 'text-room-fav'
              : ''}"
          >
            {tagEditStatus}
          </span>
          <button
            type="submit"
            disabled={tagEditPending || !tagStringDiff.trim()}
            class="inline-flex h-7 items-center gap-1.5 rounded-[3px] border border-room-accent bg-room-accent/10 px-2.5 font-mono text-[10.5px] uppercase tracking-[0.18em] text-room-accent transition-colors duration-150 hover:bg-room-accent/20 disabled:opacity-50"
          >
            {#if tagEditPending}
              <Spinner class="size-2.5 border border-room-accent/40 border-t-room-accent" />
            {/if}
            save
          </button>
        </div>
      </form>
    {/if}
  </section>

  {#each tagGroups(post, site) as [group, tags] (group)}
    <TagGroup {group} {tags} padding="tight" onTagClick={onSearchTag} />
  {/each}
</aside>
