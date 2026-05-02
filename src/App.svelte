<script lang="ts">
  import { onMount } from "svelte";
  import AccountDialog from "./components/AccountDialog.svelte";
  import OriginalViewer from "./components/OriginalViewer.svelte";
  import PostGrid from "./components/PostGrid.svelte";
  import PostSidebar from "./components/PostSidebar.svelte";
  import SearchHeader from "./components/SearchHeader.svelte";
  import Toolbar from "./components/Toolbar.svelte";
  import {
    createComment,
    downloadFile,
    favoritePost,
    fetchComments,
    fetchPreview,
    getAccount,
    hideComment,
    mediaUrl,
    originalUrl,
    searchPosts,
    signIn,
    signOutAccount,
    thumbnailUrl,
    unfavoritePost,
    updatePostTags,
  } from "./lib/e621";
  import { queryWithSort, sortModeFromQuery } from "./lib/search";
  import type {
    CommentState,
    OriginalViewer as OriginalViewerState,
    Post,
    Preset,
    SortMode,
  } from "./lib/types";

  const basePresets: Preset[] = [
    { label: "Hot", value: "order:rank" },
    { label: "Popular Today", value: "date:day order:score" },
  ];

  let query = $state(basePresets[0].value);
  let status = $state("idle");
  let posts = $state<Post[]>([]);
  let selectedId = $state<number | null>(null);
  let loading = $state(false);
  let hasSearched = $state(false);
  let previews = $state<Record<number, string>>({});
  let failedPreviews = $state<Record<number, boolean>>({});
  let activePreset = $state<string | null>(basePresets[0].value);
  let sortMode = $state<SortMode>("latest");
  let page = $state(1);
  let hasNextPage = $state(false);

  // Mirrors PostGrid CSS: p-3 (12px), gap-2 (8px), minmax(176px,1fr), h-16 metadata strip (64px).
  function computePageSize(): number {
    const grid = document.querySelector<HTMLElement>("[data-grid-scroll]");
    if (!grid) return 64;
    const padding = 12;
    const gap = 8;
    const tileMin = 176;
    const metaH = 64;
    const w = grid.clientWidth - 2 * padding;
    const h = grid.clientHeight - 2 * padding;
    if (w <= 0 || h <= 0) return 64;
    const cols = Math.max(1, Math.floor((w + gap) / (tileMin + gap)));
    const tileW = (w - gap * (cols - 1)) / cols;
    const tileH = tileW + metaH;
    const rows = Math.max(1, Math.floor((h + gap) / (tileH + gap)));
    return Math.min(320, Math.max(8, cols * rows));
  }

  let username = $state<string | null>(null);
  let showAccount = $state(false);
  let usernameInput = $state("");
  let apiKeyInput = $state("");
  let accountStatus = $state("");
  let accountSaving = $state(false);
  let favoritePending = $state<Record<number, boolean>>({});
  let downloadPending = $state<Record<number, boolean>>({});
  let downloadStatus = $state("");
  let originalViewer = $state<OriginalViewerState | null>(null);
  let imageOnly = $state(false);
  let comments = $state<CommentState>({
    items: [],
    loading: false,
    error: null,
    body: "",
    submitting: false,
    submitError: null,
    hiding: {},
  });

  let selectedPost = $derived(posts.find((post) => post.id === selectedId) ?? null);
  let presets = $derived(
    username
      ? [...basePresets, { label: "Favorites", value: `fav:${username}` }]
      : [...basePresets, { label: "Favorites", value: "", requiresAccount: true }],
  );

  onMount(() => {
    void loadAccount();
    window.addEventListener("keydown", onWindowKeydown);
    window.addEventListener("popstate", onPopState);
    window.addEventListener("mouseup", onMouseNavButton);
    return () => {
      window.removeEventListener("keydown", onWindowKeydown);
      window.removeEventListener("popstate", onPopState);
      window.removeEventListener("mouseup", onMouseNavButton);
    };
  });

  function onWindowKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      closeTopLayer();
      return;
    }

    if (event.key !== "ArrowLeft" && event.key !== "ArrowRight") return;
    if (showAccount || isTextInput(event.target)) return;
    event.preventDefault();
    moveSelection(event.key === "ArrowRight" ? 1 : -1, !!originalViewer);
  }

  function isTextInput(target: EventTarget | null) {
    if (!(target instanceof HTMLElement)) return false;
    return (
      target.isContentEditable ||
      target.tagName === "INPUT" ||
      target.tagName === "TEXTAREA" ||
      target.tagName === "SELECT"
    );
  }

  function moveSelection(delta: number, openViewer = false) {
    if (posts.length === 0) return;
    const currentIndex = selectedId === null ? -1 : posts.findIndex((post) => post.id === selectedId);
    const fallbackIndex = delta > 0 ? 0 : posts.length - 1;
    const nextIndex =
      currentIndex === -1
        ? fallbackIndex
        : Math.min(posts.length - 1, Math.max(0, currentIndex + delta));
    const nextPost = posts[nextIndex];
    selectedId = nextPost?.id ?? null;
    if (openViewer && nextPost) {
      void openOriginal(nextPost, "replace");
    }
  }

  function onPopState(event: PopStateEvent) {
    const state = event.state as { viewer?: number } | null;
    const viewerId = state?.viewer;
    if (typeof viewerId === "number") {
      if (originalViewer?.post.id === viewerId) return;
      const post = posts.find((p) => p.id === viewerId);
      if (post) {
        void openOriginal(post, "skip");
      } else {
        originalViewer = null;
        imageOnly = false;
      }
    } else {
      closeTopLayer(true);
    }
  }

  function onMouseNavButton(event: MouseEvent) {
    if (event.button === 3) {
      closeTopLayer();
    } else if (event.button === 4) {
      history.forward();
    }
  }

  function closeTopLayer(fromHistory = false) {
    if (imageOnly) {
      imageOnly = false;
    } else if (originalViewer) {
      closeOriginal(fromHistory);
    } else if (showAccount && !accountSaving) {
      closeAccount();
    }
  }

  function onBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      closeAccount();
    }
  }

  async function loadAccount() {
    try {
      const result = await getAccount();
      username = result.username;
    } catch {
      username = null;
    }
  }

  async function search(targetPage: number = 1) {
    const tags = activePreset ? query.trim() : queryWithSort(query, sortMode);
    const limit = computePageSize();
    page = Math.max(1, targetPage);
    loading = true;
    hasSearched = true;
    status = `searching ${tags || "all"}`;
    posts = [];
    previews = {};
    failedPreviews = {};
    selectedId = null;
    hasNextPage = false;

    try {
      const result = await searchPosts(tags, page, limit);
      posts = result.posts;
      hasNextPage = posts.length >= limit;
      status = `${posts.length} post${posts.length === 1 ? "" : "s"}`;
      for (const post of posts) {
        void loadPreview(post);
      }
    } catch (error) {
      status = `error: ${String(error)}`;
    } finally {
      loading = false;
    }
  }

  function goToPage(delta: number) {
    const next = page + delta;
    if (next < 1 || (delta > 0 && !hasNextPage) || loading) return;
    void search(next);
    document.querySelector("[data-grid-scroll]")?.scrollTo({ top: 0 });
  }

  async function loadPreview(post: Post) {
    const url = thumbnailUrl(post);
    if (!url) {
      markPreviewFailed(post.id);
      return;
    }

    try {
      const result = await fetchPreview(url);
      previews[post.id] = result.data_url;
    } catch {
      markPreviewFailed(post.id);
    }
  }

  function markPreviewFailed(postId: number) {
    delete previews[postId];
    previews = { ...previews };
    failedPreviews[postId] = true;
    failedPreviews = { ...failedPreviews };
  }

  function applyPreset(preset: Preset) {
    if (preset.requiresAccount) {
      openAccount();
      return;
    }

    query = preset.value;
    activePreset = preset.value;
    sortMode = sortModeFromQuery(preset.value);
    void search();
  }

  function applySort(value: SortMode) {
    sortMode = value;
    activePreset = null;
    if (hasSearched || query.trim()) {
      void search();
    }
  }

  function setQuery(value: string) {
    query = value;
    activePreset = null;
  }

  async function openOriginal(post: Post, historyMode: "push" | "replace" | "skip" = "push") {
    const url = originalUrl(post);
    downloadStatus = "";
    imageOnly = false;
    comments = {
      items: [],
      loading: true,
      error: null,
      body: "",
      submitting: false,
      submitError: null,
      hiding: {},
    };
    originalViewer = {
      post,
      dataUrl: null,
      loading: !!url,
      error: url ? null : "original file is unavailable",
    };
    if (historyMode === "push") {
      history.pushState({ viewer: post.id }, "", "");
    } else if (historyMode === "replace") {
      history.replaceState({ viewer: post.id }, "", "");
    }

    void loadComments(post.id);

    if (!url) return;

    try {
      const result = await mediaUrl(url);
      if (originalViewer?.post.id === post.id) {
        originalViewer = {
          ...originalViewer,
          dataUrl: result,
          loading: false,
          error: null,
        };
      }
    } catch (error) {
      if (originalViewer?.post.id === post.id) {
        originalViewer = {
          ...originalViewer,
          dataUrl: null,
          loading: false,
          error: String(error),
        };
      }
    }
  }

  function closeOriginal(fromHistory = false) {
    originalViewer = null;
    imageOnly = false;
    comments = {
      items: [],
      loading: false,
      error: null,
      body: "",
      submitting: false,
      submitError: null,
      hiding: {},
    };
    if (!fromHistory && history.state?.viewer) {
      history.back();
    }
  }

  async function loadComments(postId: number) {
    comments = {
      ...comments,
      loading: true,
      error: null,
    };

    try {
      const items = await fetchComments(postId);
      if (originalViewer?.post.id === postId) {
        comments = {
          ...comments,
          items,
          loading: false,
          error: null,
        };
      }
    } catch (error) {
      if (originalViewer?.post.id === postId) {
        comments = {
          ...comments,
          items: [],
          loading: false,
          error: String(error),
        };
      }
    }
  }

  function setCommentBody(value: string) {
    comments = {
      ...comments,
      body: value,
      submitError: null,
    };
  }

  async function submitComment(post: Post) {
    if (!username) {
      openAccount();
      return;
    }
    const body = comments.body.trim();
    if (!body || comments.submitting) return;

    comments = {
      ...comments,
      submitting: true,
      submitError: null,
    };

    try {
      const created = await createComment(post.id, body);
      if (originalViewer?.post.id === post.id) {
        const nextPost = {
          ...originalViewer.post,
          comment_count: (originalViewer.post.comment_count ?? comments.items.length) + 1,
        };
        originalViewer = {
          ...originalViewer,
          post: nextPost,
        };
        posts = posts.map((p) =>
          p.id === post.id ? { ...p, comment_count: (p.comment_count ?? 0) + 1 } : p,
        );
        comments = {
          ...comments,
          items: [...comments.items, created],
          body: "",
          submitting: false,
          submitError: null,
        };
      }
    } catch (error) {
      comments = {
        ...comments,
        submitting: false,
        submitError: String(error),
      };
    }
  }

  async function updateTags(post: Post, tagStringDiff: string, editReason: string) {
    if (!username) {
      openAccount();
      return;
    }
    const updated = await updatePostTags(post.id, tagStringDiff, editReason);
    posts = posts.map((p) => (p.id === post.id ? updated : p));
    if (originalViewer?.post.id === post.id) {
      originalViewer = {
        ...originalViewer,
        post: updated,
      };
    }
  }

  async function hideOwnComment(commentId: number) {
    if (!username || comments.hiding[commentId]) return;
    comments = {
      ...comments,
      submitError: null,
      hiding: {
        ...comments.hiding,
        [commentId]: true,
      },
    };
    try {
      const hidden = await hideComment(commentId);
      comments = {
        ...comments,
        items: comments.items.map((comment) =>
          comment.id === commentId ? { ...comment, ...hidden, is_hidden: true } : comment,
        ),
        hiding: {
          ...comments.hiding,
          [commentId]: false,
        },
      };
    } catch (error) {
      comments = {
        ...comments,
        submitError: String(error),
        hiding: {
          ...comments.hiding,
          [commentId]: false,
        },
      };
    }
  }

  function searchTag(tag: string) {
    closeOriginal();
    query = tag;
    activePreset = null;
    void search();
  }

  function openAccount() {
    usernameInput = username ?? "";
    apiKeyInput = "";
    accountStatus = "";
    showAccount = true;
  }

  function closeAccount() {
    if (accountSaving) return;
    showAccount = false;
    accountStatus = "";
  }

  async function submitSignIn(event: Event) {
    event.preventDefault();
    const u = usernameInput.trim();
    const k = apiKeyInput.trim();
    if (!u || !k) {
      accountStatus = "username and api key are required";
      return;
    }
    accountSaving = true;
    accountStatus = "verifying";
    try {
      const result = await signIn(u, k);
      username = result.username;
      apiKeyInput = "";
      showAccount = false;
      accountStatus = "";
    } catch (error) {
      accountStatus = String(error);
    } finally {
      accountSaving = false;
    }
  }

  async function signOut() {
    accountSaving = true;
    accountStatus = "";
    try {
      await signOutAccount();
      username = null;
      apiKeyInput = "";
      showAccount = false;
      if (activePreset?.startsWith("fav:")) {
        applyPreset(basePresets[0]);
      }
    } catch (error) {
      accountStatus = String(error);
    } finally {
      accountSaving = false;
    }
  }

  async function toggleFavorite(post: Post) {
    if (!username) {
      openAccount();
      return;
    }
    if (favoritePending[post.id]) return;

    const wasFavorited = post.is_favorited === true;
    favoritePending[post.id] = true;

    try {
      if (wasFavorited) {
        await unfavoritePost(post.id);
      } else {
        await favoritePost(post.id);
      }
      posts = posts.map((p) =>
        p.id === post.id
          ? {
              ...p,
              is_favorited: !wasFavorited,
              fav_count: Math.max(0, (p.fav_count ?? 0) + (wasFavorited ? -1 : 1)),
            }
          : p,
      );
      if (originalViewer?.post.id === post.id) {
        originalViewer = {
          ...originalViewer,
          post: {
            ...originalViewer.post,
            is_favorited: !wasFavorited,
            fav_count: Math.max(
              0,
              (originalViewer.post.fav_count ?? 0) + (wasFavorited ? -1 : 1),
            ),
          },
        };
      }
    } catch (error) {
      status = `favorite failed: ${String(error)}`;
    } finally {
      delete favoritePending[post.id];
      favoritePending = { ...favoritePending };
    }
  }

  async function downloadOriginal(post: Post) {
    const url = originalUrl(post);
    if (!url || downloadPending[post.id]) return;

    downloadPending[post.id] = true;
    downloadPending = { ...downloadPending };
    downloadStatus = "downloading";

    try {
      const artist = post.tags?.artist?.[0] || "unknown_artist";
      const ext = post.file?.ext || "jpg";
      const filename = `${artist}_${post.id}.${ext}`;
      const path = await downloadFile(url, filename);
      downloadStatus = `saved ${path}`;
    } catch (error) {
      downloadStatus = `download failed: ${String(error)}`;
    } finally {
      delete downloadPending[post.id];
      downloadPending = { ...downloadPending };
    }
  }
</script>

<svelte:head>
  <title>Clowder</title>
</svelte:head>

<main class="grid h-screen grid-rows-[48px_36px_minmax(0,1fr)] bg-room-bg text-room-text">
  <SearchHeader
    {query}
    {status}
    {loading}
    {username}
    {hasSearched}
    {page}
    {hasNextPage}
    onQueryChange={setQuery}
    onSearch={search}
    onOpenAccount={openAccount}
    onPageChange={goToPage}
  />

  <Toolbar
    {presets}
    {activePreset}
    {sortMode}
    onApplyPreset={applyPreset}
    onApplySort={applySort}
  />

  <section class="grid min-h-0 grid-cols-[300px_minmax(0,1fr)]">
    <PostSidebar
      {selectedPost}
      {username}
      {favoritePending}
      onOpenOriginal={openOriginal}
      onToggleFavorite={toggleFavorite}
    />
    <PostGrid
      {posts}
      {loading}
      {hasSearched}
      {selectedId}
      {previews}
      {failedPreviews}
      onSelect={(id) => (selectedId = id)}
      onOpenOriginal={openOriginal}
      onPreviewError={markPreviewFailed}
    />
  </section>

  {#if originalViewer}
    <OriginalViewer
      viewer={originalViewer}
      {imageOnly}
      {username}
      favoritePending={!!favoritePending[originalViewer.post.id]}
      downloadPending={!!downloadPending[originalViewer.post.id]}
      {downloadStatus}
      {comments}
      onClose={closeOriginal}
      onToggleImageOnly={() => (imageOnly = !imageOnly)}
      onSearchTag={searchTag}
      onToggleFavorite={toggleFavorite}
      onDownload={downloadOriginal}
      onCommentBodyChange={setCommentBody}
      onSubmitComment={submitComment}
      onRefreshComments={loadComments}
      onOpenAccount={openAccount}
      onUpdateTags={updateTags}
      onHideComment={hideOwnComment}
    />
  {/if}

  {#if showAccount}
    <AccountDialog
      {username}
      {usernameInput}
      {apiKeyInput}
      {accountStatus}
      {accountSaving}
      onClose={closeAccount}
      onBackdropClick={onBackdropClick}
      onSubmit={submitSignIn}
      onSignOut={signOut}
      onUsernameInput={(value) => (usernameInput = value)}
      onApiKeyInput={(value) => (apiKeyInput = value)}
    />
  {/if}
</main>
