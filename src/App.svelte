<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import OriginalViewer from "./components/OriginalViewer.svelte";
  import PostGrid from "./components/PostGrid.svelte";
  import PostSidebar from "./components/PostSidebar.svelte";
  import SearchHeader from "./components/SearchHeader.svelte";
  import SettingsDialog from "./components/SettingsDialog.svelte";
  import Toasts from "./components/Toasts.svelte";
  import Toolbar from "./components/Toolbar.svelte";
  import { accountStore } from "./lib/account-store.svelte";
  import { downloadStore } from "./lib/download-store.svelte";
  import { isTextInput } from "./lib/keyboard";
  import { postActionsStore } from "./lib/post-actions-store.svelte";
  import { searchHistoryStore } from "./lib/search-history-store.svelte";
  import { searchStore } from "./lib/search-store.svelte";
  import { settingsStore } from "./lib/settings-store.svelte";
  import { toastStore } from "./lib/toast-store.svelte";
  import { updateStore } from "./lib/update-store.svelte";
  import { viewerStore } from "./lib/viewer-store.svelte";
  import type { Post, Preset, SettingsSection, SortMode } from "./lib/types";

  let showSettings = $state(false);
  let settingsSection = $state<SettingsSection>("account");
  let usernameInput = $state("");
  let apiKeyInput = $state("");

  let systemPrefersLight = $state(false);
  let systemPrefersReducedMotion = $state(false);

  let selectedPost = $derived(
    searchStore.posts.find((post) => post.id === searchStore.selectedId) ?? null,
  );
  let presets = $derived(
    accountStore.username
      ? [...searchStore.basePresets, { label: "Favorites", value: `fav:${accountStore.username}` }]
      : [...searchStore.basePresets, { label: "Favorites", value: "", requiresAccount: true }],
  );

  // Mirrors PostGrid CSS: p-3 (12px), gap-2 (8px), minmax(var(--clowder-tile-min),1fr), h-16 metadata strip (64px).
  function computePageSize(): number {
    const grid = document.querySelector<HTMLElement>("[data-grid-scroll]");
    if (!grid) return 64;
    const padding = 12;
    const gap = 8;
    const tileMin = settingsStore.current.appearance.grid_min_tile_px;
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

  function search(targetPage: number = 1) {
    if (targetPage === 1) {
      searchHistoryStore.push(searchStore.query);
    }
    return searchStore.search(targetPage, computePageSize());
  }

  onMount(() => {
    void accountStore.load();
    void settingsStore.load();
    const updateCheckTimer = window.setTimeout(() => {
      void updateStore.check(true);
    }, 3000);
    window.addEventListener("keydown", onWindowKeydown);
    window.addEventListener("popstate", onPopState);
    window.addEventListener("mouseup", onMouseNavButton);

    let unlistenAuthExpired: (() => void) | undefined;
    void listen("auth-expired", () => {
      accountStore.expire();
      toastStore.error("Your e621 session expired. Sign in again.");
      if (searchStore.activePreset?.startsWith("fav:")) {
        applyPreset(searchStore.basePresets[0]);
      }
    }).then((unlisten) => {
      unlistenAuthExpired = unlisten;
    });

    const lightMq = window.matchMedia("(prefers-color-scheme: light)");
    const motionMq = window.matchMedia("(prefers-reduced-motion: reduce)");
    systemPrefersLight = lightMq.matches;
    systemPrefersReducedMotion = motionMq.matches;
    const onLight = (event: MediaQueryListEvent) => (systemPrefersLight = event.matches);
    const onMotion = (event: MediaQueryListEvent) => (systemPrefersReducedMotion = event.matches);
    lightMq.addEventListener("change", onLight);
    motionMq.addEventListener("change", onMotion);

    return () => {
      window.clearTimeout(updateCheckTimer);
      window.removeEventListener("keydown", onWindowKeydown);
      window.removeEventListener("popstate", onPopState);
      window.removeEventListener("mouseup", onMouseNavButton);
      lightMq.removeEventListener("change", onLight);
      motionMq.removeEventListener("change", onMotion);
      unlistenAuthExpired?.();
    };
  });

  $effect(() => {
    const appearance = settingsStore.current.appearance;
    const root = document.documentElement;

    const resolvedTheme =
      appearance.theme === "system" ? (systemPrefersLight ? "light" : "dark") : appearance.theme;
    if (resolvedTheme === "light") {
      root.dataset.theme = "light";
    } else {
      delete root.dataset.theme;
    }

    if (appearance.motion === "system") {
      delete root.dataset.motion;
    } else {
      root.dataset.motion = appearance.motion;
    }

    root.style.setProperty("--clowder-tile-min", `${appearance.grid_min_tile_px}px`);
  });

  function onWindowKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      closeTopLayer();
      return;
    }

    if (event.key !== "ArrowLeft" && event.key !== "ArrowRight") return;
    if (showSettings || isTextInput(event.target)) return;
    event.preventDefault();
    const next = searchStore.moveSelection(event.key === "ArrowRight" ? 1 : -1);
    if (viewerStore.viewer && next) {
      openOriginal(next, "replace");
    }
  }

  function onPopState(event: PopStateEvent) {
    const state = event.state as { viewer?: number } | null;
    const viewerId = state?.viewer;
    if (typeof viewerId === "number") {
      if (viewerStore.viewer?.post.id === viewerId) return;
      const post = searchStore.postById(viewerId);
      if (post) {
        openOriginal(post, "skip");
      } else {
        viewerStore.close(true);
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
    if (viewerStore.imageOnly) {
      viewerStore.setImageOnly(false);
    } else if (viewerStore.viewer) {
      viewerStore.close(fromHistory);
    } else if (showSettings && !accountStore.saving) {
      closeSettings();
    }
  }

  function onBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      closeSettings();
    }
  }

  function goToPage(delta: number) {
    const next = searchStore.page + delta;
    if (next < 1 || (delta > 0 && !searchStore.hasNextPage) || searchStore.loading) return;
    void search(next);
    document.querySelector("[data-grid-scroll]")?.scrollTo({ top: 0 });
  }

  function applyPreset(preset: Preset) {
    if (preset.requiresAccount) {
      openSettings("account");
      return;
    }
    searchStore.applyPreset(preset);
    void search();
  }

  function applySort(value: SortMode) {
    searchStore.applySort(value);
    if (searchStore.hasSearched || searchStore.query.trim()) {
      void search();
    }
  }

  function searchTag(tag: string) {
    viewerStore.close();
    searchStore.setQueryForTag(tag);
    void search();
  }

  function openOriginal(post: Post, historyMode: "push" | "replace" | "skip" = "push") {
    downloadStore.reset();
    void viewerStore.open(post, historyMode);
  }

  async function toggleFavorite(post: Post) {
    if (!accountStore.username) {
      openSettings("account");
      return;
    }
    const error = await postActionsStore.toggleFavorite(post);
    if (error) {
      const message = `favorite failed: ${error}`;
      searchStore.status = message;
      toastStore.error(message);
    }
  }

  async function updateTags(post: Post, tagStringDiff: string, editReason: string) {
    if (!accountStore.username) {
      openSettings("account");
      return;
    }
    await postActionsStore.updateTags(post, tagStringDiff, editReason);
  }

  async function submitComment(post: Post) {
    if (!accountStore.username) {
      openSettings("account");
      return;
    }
    const newCount = await viewerStore.submitComment(post);
    if (newCount !== null) {
      searchStore.updatePost(post.id, { comment_count: newCount });
    }
  }

  async function hideOwnComment(commentId: number) {
    if (!accountStore.username) return;
    await viewerStore.hideOwnComment(commentId);
  }

  function openSettings(section: SettingsSection = "account") {
    usernameInput = accountStore.username ?? "";
    apiKeyInput = "";
    accountStore.status = "";
    settingsSection = section;
    showSettings = true;
  }

  function closeSettings() {
    if (accountStore.saving) return;
    showSettings = false;
    accountStore.status = "";
  }

  async function submitSignIn(event: Event) {
    event.preventDefault();
    const u = usernameInput.trim();
    const k = apiKeyInput.trim();
    if (!u || !k) {
      accountStore.status = "username and api key are required";
      return;
    }
    const ok = await accountStore.signIn(u, k);
    if (ok) {
      apiKeyInput = "";
      showSettings = false;
    }
  }

  async function signOut() {
    const ok = await accountStore.signOut();
    if (ok) {
      apiKeyInput = "";
      showSettings = false;
      if (searchStore.activePreset?.startsWith("fav:")) {
        applyPreset(searchStore.basePresets[0]);
      }
    }
  }
</script>

<svelte:head>
  <title>Clowder</title>
</svelte:head>

<main class="grid h-screen grid-rows-[48px_36px_minmax(0,1fr)] bg-room-bg text-room-text">
  <SearchHeader
    query={searchStore.query}
    status={searchStore.status}
    loading={searchStore.loading}
    username={accountStore.username}
    hasSearched={searchStore.hasSearched}
    page={searchStore.page}
    hasNextPage={searchStore.hasNextPage}
    onQueryChange={(value) => searchStore.setQuery(value)}
    onSearch={() => search()}
    onOpenAccount={() => openSettings("account")}
    onOpenSettings={() => openSettings("downloads")}
    onPageChange={goToPage}
  />

  <Toolbar
    {presets}
    activePreset={searchStore.activePreset}
    sortMode={searchStore.sortMode}
    onApplyPreset={applyPreset}
    onApplySort={applySort}
  />

  <section class="grid min-h-0 grid-cols-[300px_minmax(0,1fr)]">
    <PostSidebar
      {selectedPost}
      username={accountStore.username}
      favoritePending={postActionsStore.favoritePending}
      onOpenOriginal={openOriginal}
      onToggleFavorite={toggleFavorite}
    />
    <PostGrid
      posts={searchStore.posts}
      loading={searchStore.loading}
      hasSearched={searchStore.hasSearched}
      selectedId={searchStore.selectedId}
      previews={searchStore.previews}
      failedPreviews={searchStore.failedPreviews}
      onSelect={(id) => searchStore.selectId(id)}
      onOpenOriginal={openOriginal}
      onPreviewError={(id) => searchStore.markPreviewFailed(id)}
    />
  </section>

  {#if viewerStore.viewer}
    <OriginalViewer
      viewer={viewerStore.viewer}
      imageOnly={viewerStore.imageOnly}
      username={accountStore.username}
      favoritePending={postActionsStore.isFavoritePending(viewerStore.viewer.post.id)}
      downloadPending={downloadStore.isPending(viewerStore.viewer.post.id)}
      downloadStatus={downloadStore.status}
      comments={viewerStore.comments}
      onClose={() => viewerStore.close()}
      onToggleImageOnly={() => viewerStore.toggleImageOnly()}
      onSearchTag={searchTag}
      onToggleFavorite={toggleFavorite}
      onDownload={(post) => downloadStore.download(post)}
      onCommentBodyChange={(body) => viewerStore.setCommentBody(body)}
      onSubmitComment={submitComment}
      onRefreshComments={(id) => viewerStore.loadComments(id)}
      onOpenAccount={() => openSettings("account")}
      onUpdateTags={updateTags}
      onHideComment={hideOwnComment}
    />
  {/if}

  <Toasts />

  {#if showSettings}
    <SettingsDialog
      username={accountStore.username}
      initialSection={settingsSection}
      {usernameInput}
      {apiKeyInput}
      accountStatus={accountStore.status}
      accountSaving={accountStore.saving}
      onClose={closeSettings}
      {onBackdropClick}
      onSubmit={submitSignIn}
      onSignOut={signOut}
      onUsernameInput={(value) => (usernameInput = value)}
      onApiKeyInput={(value) => (apiKeyInput = value)}
    />
  {/if}
</main>
