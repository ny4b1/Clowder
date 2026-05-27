<script lang="ts">
  import type { CommentState, Post } from "../lib/types";

  type Props = {
    post: Post;
    username: string | null;
    favoritePending: boolean;
    downloadPending: boolean;
    downloadStatus: string;
    comments: CommentState;
    onToggleFavorite: (post: Post) => void;
    onDownload: (post: Post) => void;
    onCommentBodyChange: (body: string) => void;
    onSubmitComment: (post: Post) => void;
    onRefreshComments: (postId: number) => void;
    onOpenAccount: () => void;
    onHideComment: (commentId: number) => void;
  };

  let {
    post,
    username,
    favoritePending,
    downloadPending,
    downloadStatus,
    comments,
    onToggleFavorite,
    onDownload,
    onCommentBodyChange,
    onSubmitComment,
    onRefreshComments,
    onOpenAccount,
    onHideComment,
  }: Props = $props();

  function commentDate(value: string | null | undefined) {
    if (!value) return "";
    const date = new Date(value);
    if (Number.isNaN(date.getTime())) return "";
    return date.toLocaleString(undefined, {
      year: "numeric",
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  }

  function commentAuthor(value: string | null | undefined) {
    return value?.trim() || "anonymous";
  }

  function canHideComment(commentCreator: string | null | undefined) {
    return !!username && commentCreator === username;
  }
</script>

<section class="grid min-h-0 grid-rows-[auto_minmax(0,1fr)] border-t border-room-line bg-room-panel/25">
  <div class="flex flex-wrap items-center gap-2 border-b border-room-line px-4 py-3">
    <button
      type="button"
      onclick={() => onToggleFavorite(post)}
      disabled={favoritePending}
      class="inline-flex h-8 items-center gap-1.5 rounded-[3px] border bg-room-panel px-3 font-mono text-[10.5px] uppercase tracking-[0.18em] transition-colors duration-150 disabled:opacity-50 {post.is_favorited
        ? 'border-room-fav text-room-fav hover:bg-room-fav/10'
        : 'border-room-line-strong text-room-text-mid hover:border-room-fav hover:text-room-fav'}"
    >
      <svg
        class="size-3"
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
      {post.is_favorited ? "favorited" : "favorite"}
    </button>

    <button
      type="button"
      onclick={() => onDownload(post)}
      disabled={downloadPending || !post.file?.url}
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
      <span class="font-mono text-[10px] text-room-text-low">sign in to favorite or comment</span>
    {/if}
    {#if downloadStatus}
      <span class="font-mono text-[10.5px] text-room-text-low">{downloadStatus}</span>
    {/if}
  </div>

  <div class="grid min-h-0 grid-cols-[minmax(0,1fr)_320px]">
    <div class="min-h-0 overflow-auto px-4 py-3">
      <div class="mb-2 flex items-center justify-between gap-3">
        <div class="font-mono text-[10px] uppercase tracking-[0.22em] text-room-text-low">
          comments
          <span class="tracking-normal text-room-text-mid">
            {post.comment_count ?? comments.items.length}
          </span>
        </div>
        <button
          type="button"
          onclick={() => onRefreshComments(post.id)}
          disabled={comments.loading}
          class="h-6 rounded-[3px] border border-room-line bg-room-panel px-2 font-mono text-[10px] uppercase tracking-[0.16em] text-room-text-mid transition-colors duration-150 hover:border-room-line-strong hover:text-room-text disabled:opacity-50"
        >
          refresh
        </button>
      </div>

      {#if comments.loading}
        <div class="flex h-16 items-center gap-2 font-mono text-[10.5px] uppercase tracking-[0.18em] text-room-text-low">
          <span
            class="size-3 animate-spin rounded-full border border-room-line-strong border-t-room-accent"
            aria-hidden="true"
          ></span>
          loading
        </div>
      {:else if comments.error}
        <p class="font-mono text-[10.5px] leading-relaxed text-room-fav">
          {comments.error}
        </p>
      {:else if comments.items.length === 0}
        <p class="text-[12px] text-room-text-low">No comments yet.</p>
      {:else}
        <div class="space-y-3">
          {#each comments.items as comment (comment.id)}
            <article class="border-b border-room-line/70 pb-3 last:border-b-0 last:pb-0">
              <div class="mb-1 flex flex-wrap items-center gap-x-2 gap-y-1 font-mono text-[10px]">
                <span class="text-room-text">{commentAuthor(comment.creator_name)}</span>
                {#if comment.created_at}
                  <span class="text-room-text-low">{commentDate(comment.created_at)}</span>
                {/if}
                {#if comment.score}
                  <span class="text-room-text-low">score {comment.score}</span>
                {/if}
                {#if comment.is_hidden}
                  <span class="text-room-fav">hidden</span>
                {/if}
                {#if comment.is_sticky}
                  <span class="text-room-accent">sticky</span>
                {/if}
                {#if canHideComment(comment.creator_name) && !comment.is_hidden}
                  <button
                    type="button"
                    onclick={() => onHideComment(comment.id)}
                    disabled={comments.hiding[comment.id]}
                    class="ml-auto font-mono text-[10px] uppercase tracking-[0.14em] text-room-text-low transition-colors duration-150 hover:text-room-fav disabled:opacity-50"
                  >
                    {comments.hiding[comment.id] ? "hiding" : "hide"}
                  </button>
                {/if}
              </div>
              <p class="whitespace-pre-wrap break-words text-[12px] leading-relaxed text-room-text-mid">
                {comment.body}
              </p>
            </article>
          {/each}
        </div>
      {/if}
    </div>

    <form
      class="grid min-h-0 grid-rows-[auto_minmax(0,1fr)_auto] border-l border-room-line px-4 py-3"
      onsubmit={(event) => {
        event.preventDefault();
        onSubmitComment(post);
      }}
    >
      <div class="mb-2 font-mono text-[10px] uppercase tracking-[0.22em] text-room-text-low">
        add comment
      </div>
      <textarea
        value={comments.body}
        oninput={(event) => onCommentBodyChange(event.currentTarget.value)}
        disabled={!username || comments.submitting}
        maxlength="2000"
        class="min-h-0 resize-none rounded-[3px] border border-room-line bg-room-panel px-2.5 py-2 text-[12px] leading-relaxed text-room-text outline-none transition-colors duration-150 placeholder:text-room-text-low focus:border-room-accent disabled:opacity-50"
        placeholder={username ? "Write a comment" : "Sign in to comment"}
      ></textarea>
      <div class="mt-2 flex items-center justify-between gap-2">
        <div class="min-w-0 font-mono text-[10px] text-room-text-low">
          {#if comments.submitError}
            <span class="text-room-fav">{comments.submitError}</span>
          {:else}
            {comments.body.length}/2000
          {/if}
        </div>
        {#if username}
          <button
            type="submit"
            disabled={comments.submitting || !comments.body.trim()}
            class="inline-flex h-8 items-center gap-1.5 rounded-[3px] border border-room-accent bg-room-accent/10 px-3 font-mono text-[10.5px] uppercase tracking-[0.18em] text-room-accent transition-colors duration-150 hover:bg-room-accent/20 disabled:opacity-50"
          >
            {#if comments.submitting}
              <span
                class="size-2.5 animate-spin rounded-full border border-room-accent/40 border-t-room-accent"
                aria-hidden="true"
              ></span>
            {/if}
            post
          </button>
        {:else}
          <button
            type="button"
            onclick={onOpenAccount}
            class="h-8 rounded-[3px] border border-room-line-strong bg-room-panel px-3 font-mono text-[10.5px] uppercase tracking-[0.18em] text-room-text-mid transition-colors duration-150 hover:border-room-accent hover:text-room-accent"
          >
            sign in
          </button>
        {/if}
      </div>
    </form>
  </div>
</section>
