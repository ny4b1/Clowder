<script lang="ts">
  import { onMount } from "svelte";
  import AccountDialog from "./components/AccountDialog.svelte";
  import OriginalViewer from "./components/OriginalViewer.svelte";
  import PostGrid from "./components/PostGrid.svelte";
  import PostSidebar from "./components/PostSidebar.svelte";
  import SearchHeader from "./components/SearchHeader.svelte";
  import Toolbar from "./components/Toolbar.svelte";
  import {
    downloadFile,
    favoritePost,
    fetchPreview,
    getAccount,
    originalUrl,
    searchPosts,
    signIn,
    signOutAccount,
    thumbnailUrl,
    unfavoritePost,
  } from "./lib/e621";
  import { queryWithSort, sortModeFromQuery } from "./lib/search";
  import type { OriginalViewer as OriginalViewerState, Post, Preset, SortMode } from "./lib/types";

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

  let selectedPost = $derived(posts.find((post) => post.id === selectedId) ?? null);
  let presets = $derived(
    username
      ? [...basePresets, { label: "Favorites", value: `fav:${username}` }]
      : [...basePresets, { label: "Favorites", value: "", requiresAccount: true }],
  );

  onMount(() => {
    void loadAccount();
    window.addEventListener("keydown", onWindowKeydown);
    window.addEventListener("popstate", onBackNavigation);
    window.addEventListener("mouseup", onMouseBackButton);
    return () => {
      window.removeEventListener("keydown", onWindowKeydown);
      window.removeEventListener("popstate", onBackNavigation);
      window.removeEventListener("mouseup", onMouseBackButton);
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
      void openOriginal(nextPost, true);
    }
  }

  function onBackNavigation() {
    closeTopLayer(true);
  }

  function onMouseBackButton(event: MouseEvent) {
    if (event.button === 3) {
      closeTopLayer();
    }
  }

  function closeTopLayer(fromHistory = false) {
    if (originalViewer) {
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

  async function search() {
    const tags = activePreset ? query.trim() : queryWithSort(query, sortMode);
    loading = true;
    hasSearched = true;
    status = `searching ${tags || "all"}`;
    posts = [];
    previews = {};
    failedPreviews = {};
    selectedId = null;

    try {
      const result = await searchPosts(tags);
      posts = result.posts;
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

  async function openOriginal(post: Post, replaceHistory = false) {
    const url = originalUrl(post);
    downloadStatus = "";
    originalViewer = {
      post,
      dataUrl: null,
      loading: !!url,
      error: url ? null : "original file is unavailable",
    };
    if (replaceHistory) {
      history.replaceState({ viewer: post.id }, "", "");
    } else {
      history.pushState({ viewer: post.id }, "", "");
    }

    if (!url) return;

    try {
      const result = await fetchPreview(url);
      if (originalViewer?.post.id === post.id) {
        originalViewer = {
          ...originalViewer,
          dataUrl: result.data_url,
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
    if (!fromHistory && history.state?.viewer) {
      history.back();
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
    onQueryChange={setQuery}
    onSearch={search}
    onOpenAccount={openAccount}
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
      {username}
      favoritePending={!!favoritePending[originalViewer.post.id]}
      downloadPending={!!downloadPending[originalViewer.post.id]}
      {downloadStatus}
      onClose={closeOriginal}
      onSearchTag={searchTag}
      onToggleFavorite={toggleFavorite}
      onDownload={downloadOriginal}
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
