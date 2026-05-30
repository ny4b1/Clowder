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
  import { appStore } from "./lib/app-store.svelte";
  import { downloadStore } from "./lib/download-store.svelte";
  import { isTextInput } from "./lib/keyboard";
  import { searchHistoryStore } from "./lib/search-history-store.svelte";
  import { settingsStore } from "./lib/settings-store.svelte";
  import { siteLabels, type Site } from "./lib/site";
  import { toastStore } from "./lib/toast-store.svelte";
  import { updateStore } from "./lib/update-store.svelte";
  import type { Post, Preset, SettingsSection, SortMode } from "./lib/types";

  let showSettings = $state(false);
  let settingsSection = $state<SettingsSection>("account");
  let usernameInput = $state("");
  let apiKeyInput = $state("");

  let systemPrefersLight = $state(false);
  let systemPrefersReducedMotion = $state(false);

  let selectedPost = $derived(
    appStore.search.posts.find((post) => post.id === appStore.search.selectedId) ?? null,
  );
  let presets = $derived(
    appStore.account.username
      ? [
          ...appStore.search.basePresets,
          { label: "Favorites", value: `fav:${appStore.account.username}` },
        ]
      : [...appStore.search.basePresets, { label: "Favorites", value: "", requiresAccount: true }],
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
      searchHistoryStore.push(appStore.search.query);
    }
    return appStore.search.search(targetPage, computePageSize());
  }

  function switchSite(site: Site) {
    if (site === appStore.activeSite) return;
    appStore.viewer.close();
    appStore.setSite(site);
  }

  onMount(() => {
    appStore.loadAccounts();
    void settingsStore.load();
    const updateCheckTimer = window.setTimeout(() => {
      void updateStore.check(true);
    }, 3000);
    window.addEventListener("keydown", onWindowKeydown);
    window.addEventListener("popstate", onPopState);
    window.addEventListener("mouseup", onMouseNavButton);

    let unlistenAuthExpired: (() => void) | undefined;
    void listen<Site>("auth-expired", (event) => {
      const site = event.payload;
      const ctx = appStore.contextFor(site);
      ctx.account.expire();
      toastStore.error(`Your ${siteLabels[site]} session expired. Sign in again.`);
      if (ctx.search.activePreset?.startsWith("fav:")) {
        ctx.search.applyPreset(ctx.search.basePresets[0]);
        if (site === appStore.activeSite) void search();
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
    const next = appStore.search.moveSelection(event.key === "ArrowRight" ? 1 : -1);
    if (appStore.viewer.viewer && next) {
      openOriginal(next, "replace");
    }
  }

  function onPopState(event: PopStateEvent) {
    const state = event.state as { viewer?: number } | null;
    const viewerId = state?.viewer;
    if (typeof viewerId === "number") {
      if (appStore.viewer.viewer?.post.id === viewerId) return;
      const post = appStore.search.postById(viewerId);
      if (post) {
        openOriginal(post, "skip");
      } else {
        appStore.viewer.close(true);
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
    if (appStore.viewer.imageOnly) {
      appStore.viewer.setImageOnly(false);
    } else if (appStore.viewer.viewer) {
      appStore.viewer.close(fromHistory);
    } else if (showSettings && !appStore.account.saving) {
      closeSettings();
    }
  }

  function onBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      closeSettings();
    }
  }

  function goToPage(delta: number) {
    const next = appStore.search.page + delta;
    if (next < 1 || (delta > 0 && !appStore.search.hasNextPage) || appStore.search.loading) return;
    void search(next);
    document.querySelector("[data-grid-scroll]")?.scrollTo({ top: 0 });
  }

  function applyPreset(preset: Preset) {
    if (preset.requiresAccount) {
      openSettings("account");
      return;
    }
    appStore.search.applyPreset(preset);
    void search();
  }

  function applySort(value: SortMode) {
    appStore.search.applySort(value);
    if (appStore.search.hasSearched || appStore.search.query.trim()) {
      void search();
    }
  }

  function searchTag(tag: string) {
    appStore.viewer.close();
    appStore.search.setQueryForTag(tag);
    void search();
  }

  function openOriginal(post: Post, historyMode: "push" | "replace" | "skip" = "push") {
    downloadStore.reset();
    void appStore.viewer.open(post, historyMode);
  }

  async function toggleFavorite(post: Post) {
    if (!appStore.account.username) {
      openSettings("account");
      return;
    }
    const error = await appStore.postActions.toggleFavorite(post);
    if (error) {
      const message = `favorite failed: ${error}`;
      appStore.search.status = message;
      toastStore.error(message);
    }
  }

  async function updateTags(post: Post, tagStringDiff: string, editReason: string) {
    if (!appStore.account.username) {
      openSettings("account");
      return;
    }
    await appStore.postActions.updateTags(post, tagStringDiff, editReason);
  }

  async function submitComment(post: Post) {
    if (!appStore.account.username) {
      openSettings("account");
      return;
    }
    const newCount = await appStore.viewer.submitComment(post);
    if (newCount !== null) {
      appStore.search.updatePost(post.id, { comment_count: newCount });
    }
  }

  async function hideOwnComment(commentId: number) {
    if (!appStore.account.username) return;
    await appStore.viewer.hideOwnComment(commentId);
  }

  function openSettings(section: SettingsSection = "account") {
    usernameInput = appStore.account.username ?? "";
    apiKeyInput = "";
    appStore.account.status = "";
    settingsSection = section;
    showSettings = true;
  }

  function closeSettings() {
    if (appStore.account.saving) return;
    showSettings = false;
    appStore.account.status = "";
  }

  async function submitSignIn(event: Event) {
    event.preventDefault();
    const u = usernameInput.trim();
    const k = apiKeyInput.trim();
    if (!u || !k) {
      appStore.account.status = "username and api key are required";
      return;
    }
    const ok = await appStore.account.signIn(u, k);
    if (ok) {
      apiKeyInput = "";
      showSettings = false;
    }
  }

  async function signOut() {
    const ok = await appStore.account.signOut();
    if (ok) {
      apiKeyInput = "";
      showSettings = false;
      if (appStore.search.activePreset?.startsWith("fav:")) {
        applyPreset(appStore.search.basePresets[0]);
      }
    }
  }
</script>

<svelte:head>
  <title>Clowder</title>
</svelte:head>

<main class="grid h-screen grid-rows-[48px_36px_minmax(0,1fr)] bg-room-bg text-room-text">
  <SearchHeader
    site={appStore.activeSite}
    query={appStore.search.query}
    status={appStore.search.status}
    loading={appStore.search.loading}
    username={appStore.account.username}
    hasSearched={appStore.search.hasSearched}
    page={appStore.search.page}
    hasNextPage={appStore.search.hasNextPage}
    onSwitchSite={switchSite}
    onQueryChange={(value) => appStore.search.setQuery(value)}
    onSearch={() => search()}
    onOpenAccount={() => openSettings("account")}
    onOpenSettings={() => openSettings("downloads")}
    onPageChange={goToPage}
  />

  <Toolbar
    {presets}
    activePreset={appStore.search.activePreset}
    sortMode={appStore.search.sortMode}
    onApplyPreset={applyPreset}
    onApplySort={applySort}
  />

  <section class="grid min-h-0 grid-cols-[300px_minmax(0,1fr)]">
    <PostSidebar
      {selectedPost}
      site={appStore.activeSite}
      username={appStore.account.username}
      favoritePending={appStore.postActions.favoritePending}
      onOpenOriginal={openOriginal}
      onToggleFavorite={toggleFavorite}
    />
    <PostGrid
      posts={appStore.search.posts}
      site={appStore.activeSite}
      loading={appStore.search.loading}
      hasSearched={appStore.search.hasSearched}
      selectedId={appStore.search.selectedId}
      previews={appStore.search.previews}
      failedPreviews={appStore.search.failedPreviews}
      viewerOpen={!!appStore.viewer.viewer}
      onSelect={(id) => appStore.search.selectId(id)}
      onOpenOriginal={openOriginal}
      onPreviewError={(id) => appStore.search.markPreviewFailed(id)}
    />
  </section>

  {#if appStore.viewer.viewer}
    <OriginalViewer
      viewer={appStore.viewer.viewer}
      site={appStore.activeSite}
      imageOnly={appStore.viewer.imageOnly}
      username={appStore.account.username}
      favoritePending={appStore.postActions.isFavoritePending(appStore.viewer.viewer.post.id)}
      downloadPending={downloadStore.isPending(appStore.viewer.viewer.post.id)}
      downloadStatus={downloadStore.status}
      comments={appStore.viewer.comments}
      onClose={() => appStore.viewer.close()}
      onToggleImageOnly={() => appStore.viewer.toggleImageOnly()}
      onSearchTag={searchTag}
      onToggleFavorite={toggleFavorite}
      onDownload={(post) => downloadStore.download(post)}
      onCommentBodyChange={(body) => appStore.viewer.setCommentBody(body)}
      onSubmitComment={submitComment}
      onRefreshComments={(id) => appStore.viewer.loadComments(id)}
      onOpenAccount={() => openSettings("account")}
      onUpdateTags={updateTags}
      onHideComment={hideOwnComment}
    />
  {/if}

  <Toasts />

  {#if showSettings}
    <SettingsDialog
      site={appStore.activeSite}
      username={appStore.account.username}
      initialSection={settingsSection}
      {usernameInput}
      {apiKeyInput}
      accountStatus={appStore.account.status}
      accountSaving={appStore.account.saving}
      onClose={closeSettings}
      {onBackdropClick}
      onSubmit={submitSignIn}
      onSignOut={signOut}
      onUsernameInput={(value) => (usernameInput = value)}
      onApiKeyInput={(value) => (apiKeyInput = value)}
    />
  {/if}
</main>
