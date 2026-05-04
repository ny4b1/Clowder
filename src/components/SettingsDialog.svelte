<script lang="ts">
  import { onMount, untrack } from "svelte";
  import { getVersion } from "@tauri-apps/api/app";
  import { open as openDialog } from "@tauri-apps/plugin-dialog";
  import { dohProviderOptions } from "../lib/settings";
  import { settingsStore } from "../lib/settings-store.svelte";
  import { FILENAME_TOKENS, applyFilenameTemplate } from "../lib/template";
  import type {
    DohProvider,
    MotionPreference,
    Settings,
    SettingsSection,
    Theme,
  } from "../lib/types";

  type Props = {
    username: string | null;
    initialSection?: SettingsSection;
    usernameInput: string;
    apiKeyInput: string;
    accountStatus: string;
    accountSaving: boolean;
    onClose: () => void;
    onBackdropClick: (event: MouseEvent) => void;
    onSubmit: (event: Event) => void;
    onSignOut: () => void;
    onUsernameInput: (value: string) => void;
    onApiKeyInput: (value: string) => void;
  };

  let {
    username,
    initialSection = "account",
    usernameInput,
    apiKeyInput,
    accountStatus,
    accountSaving,
    onClose,
    onBackdropClick,
    onSubmit,
    onSignOut,
    onUsernameInput,
    onApiKeyInput,
  }: Props = $props();

  let activeSection = $state<SettingsSection>(untrack(() => initialSection));

  let pending = $state<Settings>(structuredClone($state.snapshot(settingsStore.current)));
  let saving = $state(false);
  let saveError = $state("");
  let saveStatus = $state("");

  let appVersion = $state("");

  onMount(() => {
    if (!settingsStore.loaded) {
      void settingsStore.load().then(() => {
        pending = structuredClone($state.snapshot(settingsStore.current));
      });
    } else {
      pending = structuredClone($state.snapshot(settingsStore.current));
    }
    void getVersion()
      .then((value) => (appVersion = value))
      .catch(() => (appVersion = ""));
  });

  let dirty = $derived.by(() => {
    return JSON.stringify(pending) !== JSON.stringify(settingsStore.current);
  });

  let templatePreview = $derived(
    applyFilenameTemplate(
      pending.downloads.filename_template.trim() || "{artist}_{id}.{ext}",
      {
        id: 12345,
        file: { ext: "png" },
        tags: { artist: ["sample_artist"] },
        score: { up: 100, down: 5, total: 95 },
        fav_count: 42,
      },
    ),
  );

  function setDohProvider(value: DohProvider) {
    pending.doh_provider = value;
    saveStatus = "";
  }

  function setFailClosed(value: boolean) {
    pending.fail_closed_ech = value;
    saveStatus = "";
  }

  async function pickDownloadDir() {
    try {
      const selected = await openDialog({
        directory: true,
        multiple: false,
        title: "Select download folder",
        defaultPath: pending.downloads.directory ?? undefined,
      });
      if (typeof selected === "string" && selected) {
        pending.downloads.directory = selected;
        saveStatus = "";
      }
    } catch (error) {
      saveError = String(error);
    }
  }

  function clearDownloadDir() {
    pending.downloads.directory = null;
    saveStatus = "";
  }

  function setFilenameTemplate(value: string) {
    pending.downloads.filename_template = value;
    saveStatus = "";
  }

  function insertToken(token: string) {
    pending.downloads.filename_template = pending.downloads.filename_template + token;
    saveStatus = "";
  }

  function setAutoplay(value: boolean) {
    pending.playback.autoplay = value;
    saveStatus = "";
  }

  function setRememberVolume(value: boolean) {
    pending.playback.remember_volume = value;
    saveStatus = "";
  }

  function setVideoChunk(value: number) {
    if (Number.isFinite(value) && value >= 1 && value <= 64) {
      pending.playback.video_chunk_mb = Math.round(value);
      saveStatus = "";
    }
  }

  function setTheme(value: Theme) {
    pending.appearance.theme = value;
    saveStatus = "";
  }

  function setMotion(value: MotionPreference) {
    pending.appearance.motion = value;
    saveStatus = "";
  }

  function setTilePx(value: number) {
    if (Number.isFinite(value) && value >= 120 && value <= 320) {
      pending.appearance.grid_min_tile_px = Math.round(value);
      saveStatus = "";
    }
  }

  async function saveAll() {
    if (!dirty || saving) return;
    saving = true;
    saveError = "";
    saveStatus = "saving";
    try {
      const result = await settingsStore.save($state.snapshot(pending));
      pending = structuredClone(result);
      saveStatus = "saved";
    } catch (error) {
      saveError = String(error);
      saveStatus = "";
    } finally {
      saving = false;
    }
  }

  function resetAll() {
    pending = structuredClone($state.snapshot(settingsStore.current));
    saveStatus = "";
    saveError = "";
  }

  const sections: { id: SettingsSection; label: string }[] = [
    { id: "account", label: "Account" },
    { id: "network", label: "Privacy & Network" },
    { id: "downloads", label: "Downloads" },
    { id: "playback", label: "Playback" },
    { id: "appearance", label: "Appearance" },
    { id: "about", label: "About" },
  ];

  function isBusy() {
    return accountSaving || saving;
  }

  const themeOptions: { value: Theme; label: string }[] = [
    { value: "system", label: "System" },
    { value: "dark", label: "Dark" },
    { value: "light", label: "Light" },
  ];

  const motionOptions: { value: MotionPreference; label: string; description: string }[] = [
    { value: "system", label: "System", description: "respect prefers-reduced-motion" },
    { value: "always", label: "Always animate", description: "ignore reduced motion request" },
    { value: "never", label: "Never animate", description: "disable all transitions" },
  ];

  const tilePresets = [
    { value: 144, label: "Compact" },
    { value: 176, label: "Comfortable" },
    { value: 220, label: "Spacious" },
  ];

  const chunkPresets = [
    { value: 2, label: "2 MB" },
    { value: 4, label: "4 MB" },
    { value: 8, label: "8 MB" },
    { value: 16, label: "16 MB" },
  ];
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="fixed inset-0 z-50 flex items-center justify-center bg-black/55 backdrop-blur-[2px]"
  onclick={onBackdropClick}
>
  <div
    class="grid h-[640px] max-h-[90vh] w-[760px] max-w-[92vw] grid-rows-[auto_minmax(0,1fr)] overflow-hidden rounded-[4px] border border-room-line-strong bg-room-panel-hi shadow-[0_24px_72px_rgba(0,0,0,0.6)]"
    role="dialog"
    aria-modal="true"
    aria-label="Settings"
  >
    <div class="flex items-center justify-between border-b border-room-line px-5 py-3">
      <div>
        <div class="font-mono text-[10px] uppercase tracking-[0.22em] text-room-text-low">
          settings
        </div>
        <div class="mt-0.5 text-[13px] text-room-text">
          {sections.find((s) => s.id === activeSection)?.label ?? "Settings"}
        </div>
      </div>
      <button
        type="button"
        onclick={onClose}
        disabled={isBusy()}
        class="flex size-7 items-center justify-center rounded-[3px] border border-room-line bg-room-panel text-room-text-mid transition-colors duration-150 hover:border-room-line-strong hover:text-room-text disabled:opacity-50"
        aria-label="Close"
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

    <div class="grid min-h-0 grid-cols-[180px_minmax(0,1fr)]">
      <nav class="flex flex-col border-r border-room-line bg-room-panel/40 py-3">
        {#each sections as section (section.id)}
          {@const isActive = activeSection === section.id}
          <button
            type="button"
            onclick={() => (activeSection = section.id)}
            class="relative px-5 py-2 text-left font-mono text-[11px] uppercase tracking-[0.18em] transition-colors duration-150 {isActive
              ? 'text-room-accent'
              : 'text-room-text-mid hover:text-room-text'}"
          >
            {section.label}
            {#if isActive}
              <span class="absolute bottom-1 left-5 right-5 h-px bg-room-accent" aria-hidden="true"
              ></span>
            {/if}
          </button>
        {/each}
      </nav>

      <div class="grid min-h-0 grid-rows-[minmax(0,1fr)_auto]">
        <div class="min-h-0 overflow-auto">
          {#if activeSection === "account"}
            <form onsubmit={onSubmit}>
              <div class="space-y-3 px-5 py-4">
                <label class="block">
                  <span
                    class="mb-1 block font-mono text-[10px] uppercase tracking-[0.22em] text-room-text-low"
                  >
                    username
                  </span>
                  <input
                    value={usernameInput}
                    autocomplete="username"
                    spellcheck="false"
                    oninput={(event) => onUsernameInput(event.currentTarget.value)}
                    class="block h-8 w-full rounded-[3px] border border-room-line bg-room-panel px-2.5 text-[12.5px] text-room-text outline-none transition-colors duration-150 focus:border-room-accent"
                  />
                </label>
                <label class="block">
                  <span
                    class="mb-1 block font-mono text-[10px] uppercase tracking-[0.22em] text-room-text-low"
                  >
                    api key
                  </span>
                  <input
                    value={apiKeyInput}
                    type="password"
                    autocomplete="current-password"
                    spellcheck="false"
                    oninput={(event) => onApiKeyInput(event.currentTarget.value)}
                    class="block h-8 w-full rounded-[3px] border border-room-line bg-room-panel px-2.5 font-mono text-[12px] text-room-text outline-none transition-colors duration-150 focus:border-room-accent"
                  />
                </label>
                <p class="font-mono text-[10px] leading-relaxed text-room-text-low">
                  generate at e621.net/users/home → manage api access. stored in the system
                  keychain.
                </p>
                {#if accountStatus}
                  <p
                    class="font-mono text-[10.5px] leading-relaxed {accountStatus === 'verifying'
                      ? 'text-room-text-mid'
                      : 'text-room-fav'}"
                  >
                    {accountStatus}
                  </p>
                {/if}
                <div class="flex items-center justify-between gap-2 pt-2">
                  {#if username}
                    <button
                      type="button"
                      onclick={onSignOut}
                      disabled={accountSaving}
                      class="h-8 rounded-[3px] border border-room-line bg-room-panel px-3 font-mono text-[10.5px] uppercase tracking-[0.18em] text-room-text-mid transition-colors duration-150 hover:border-room-fav hover:text-room-fav disabled:opacity-50"
                    >
                      sign out
                    </button>
                  {:else}
                    <span></span>
                  {/if}
                  <button
                    type="submit"
                    disabled={accountSaving}
                    class="flex h-8 items-center gap-1.5 rounded-[3px] border border-room-accent bg-room-accent/10 px-3 font-mono text-[10.5px] uppercase tracking-[0.18em] text-room-accent transition-colors duration-150 hover:bg-room-accent/20 disabled:opacity-50"
                  >
                    {#if accountSaving}
                      <span
                        class="size-2.5 animate-spin rounded-full border border-room-accent/40 border-t-room-accent"
                        aria-hidden="true"
                      ></span>
                    {/if}
                    {username ? "update" : "sign in"}
                  </button>
                </div>
              </div>
            </form>
          {:else if activeSection === "network"}
            <div class="space-y-5 px-5 py-4">
              <fieldset class="space-y-2">
                <legend
                  class="mb-1 block font-mono text-[10px] uppercase tracking-[0.22em] text-room-text-low"
                >
                  DoH provider
                </legend>
                <p class="mb-2 text-[11.5px] leading-relaxed text-room-text-mid">
                  DNS-over-HTTPS resolver used to fetch the e621 ECH config. The selected provider
                  sees that you connect to e621.
                </p>
                <div class="space-y-1">
                  {#each dohProviderOptions as option (option.value)}
                    {@const isSelected = pending.doh_provider === option.value}
                    <label
                      class="flex cursor-pointer items-start gap-3 rounded-[3px] border bg-room-panel px-3 py-2 transition-colors duration-150 {isSelected
                        ? 'border-room-accent'
                        : 'border-room-line hover:border-room-line-strong'}"
                    >
                      <input
                        type="radio"
                        name="doh-provider"
                        value={option.value}
                        checked={isSelected}
                        disabled={saving}
                        onchange={() => setDohProvider(option.value)}
                        class="mt-1 accent-room-accent"
                      />
                      <span class="min-w-0 flex-1">
                        <span
                          class="block text-[12.5px] {isSelected
                            ? 'text-room-accent'
                            : 'text-room-text'}"
                        >
                          {option.label}
                        </span>
                        <span
                          class="mt-0.5 block font-mono text-[10.5px] leading-relaxed text-room-text-low"
                        >
                          {option.description}
                        </span>
                      </span>
                    </label>
                  {/each}
                </div>
              </fieldset>

              <div class="space-y-2 border-t border-room-line pt-4">
                <div class="font-mono text-[10px] uppercase tracking-[0.22em] text-room-text-low">
                  ECH enforcement
                </div>
                <label
                  class="flex cursor-pointer items-start gap-3 rounded-[3px] border bg-room-panel px-3 py-2 transition-colors duration-150 {pending.fail_closed_ech
                    ? 'border-room-accent'
                    : 'border-room-line hover:border-room-line-strong'}"
                >
                  <input
                    type="checkbox"
                    checked={pending.fail_closed_ech}
                    disabled={saving}
                    onchange={(event) => setFailClosed(event.currentTarget.checked)}
                    class="mt-1 accent-room-accent"
                  />
                  <span class="min-w-0 flex-1">
                    <span class="block text-[12.5px] text-room-text">
                      Require ECH for all requests
                    </span>
                    <span
                      class="mt-0.5 block font-mono text-[10.5px] leading-relaxed text-room-text-low"
                    >
                      Refuse to connect if DoH fails or no ECH config is returned. Stronger
                      privacy, but breaks on networks that block DoH.
                    </span>
                  </span>
                </label>
              </div>
            </div>
          {:else if activeSection === "downloads"}
            <div class="space-y-5 px-5 py-4">
              <div class="space-y-2">
                <div class="font-mono text-[10px] uppercase tracking-[0.22em] text-room-text-low">
                  download folder
                </div>
                <div
                  class="flex items-stretch overflow-hidden rounded-[3px] border border-room-line bg-room-panel"
                >
                  <span
                    class="min-w-0 flex-1 truncate px-3 py-2 font-mono text-[11.5px] text-room-text"
                  >
                    {pending.downloads.directory ?? "~/Downloads/Clowder (default)"}
                  </span>
                  <button
                    type="button"
                    onclick={pickDownloadDir}
                    disabled={saving}
                    class="border-l border-room-line px-3 font-mono text-[10.5px] uppercase tracking-[0.18em] text-room-text-mid transition-colors duration-150 hover:bg-room-panel-hi hover:text-room-text disabled:opacity-50"
                  >
                    choose
                  </button>
                  {#if pending.downloads.directory}
                    <button
                      type="button"
                      onclick={clearDownloadDir}
                      disabled={saving}
                      class="border-l border-room-line px-3 font-mono text-[10.5px] uppercase tracking-[0.18em] text-room-text-mid transition-colors duration-150 hover:bg-room-panel-hi hover:text-room-text disabled:opacity-50"
                    >
                      reset
                    </button>
                  {/if}
                </div>
              </div>

              <div class="space-y-2 border-t border-room-line pt-4">
                <div class="font-mono text-[10px] uppercase tracking-[0.22em] text-room-text-low">
                  filename template
                </div>
                <input
                  value={pending.downloads.filename_template}
                  oninput={(event) => setFilenameTemplate(event.currentTarget.value)}
                  disabled={saving}
                  spellcheck="false"
                  class="block h-8 w-full rounded-[3px] border border-room-line bg-room-panel px-2.5 font-mono text-[12px] text-room-text outline-none transition-colors duration-150 focus:border-room-accent disabled:opacity-50"
                />
                <div class="flex flex-wrap gap-1">
                  {#each FILENAME_TOKENS as item (item.token)}
                    <button
                      type="button"
                      onclick={() => insertToken(item.token)}
                      disabled={saving}
                      class="inline-flex h-6 items-center rounded-[2px] border border-room-line bg-room-panel px-2 font-mono text-[10.5px] text-room-text-mid transition-colors duration-150 hover:border-room-accent hover:text-room-accent disabled:opacity-50"
                      title={item.description}
                    >
                      {item.token}
                    </button>
                  {/each}
                </div>
                <p class="font-mono text-[10.5px] leading-relaxed text-room-text-low">
                  preview: <span class="text-room-text-mid">{templatePreview}</span>
                </p>
                <p class="font-mono text-[10px] leading-relaxed text-room-text-low">
                  Existing files at the same path get a numeric suffix (-1, -2, ...) automatically.
                </p>
              </div>
            </div>
          {:else if activeSection === "playback"}
            <div class="space-y-5 px-5 py-4">
              <div class="space-y-2">
                <div class="font-mono text-[10px] uppercase tracking-[0.22em] text-room-text-low">
                  video behavior
                </div>
                <label
                  class="flex cursor-pointer items-start gap-3 rounded-[3px] border bg-room-panel px-3 py-2 transition-colors duration-150 {pending
                    .playback.autoplay
                    ? 'border-room-accent'
                    : 'border-room-line hover:border-room-line-strong'}"
                >
                  <input
                    type="checkbox"
                    checked={pending.playback.autoplay}
                    disabled={saving}
                    onchange={(event) => setAutoplay(event.currentTarget.checked)}
                    class="mt-1 accent-room-accent"
                  />
                  <span class="min-w-0 flex-1">
                    <span class="block text-[12.5px] text-room-text">Autoplay videos</span>
                    <span
                      class="mt-0.5 block font-mono text-[10.5px] leading-relaxed text-room-text-low"
                    >
                      Start playback automatically when opening a video post.
                    </span>
                  </span>
                </label>
                <label
                  class="flex cursor-pointer items-start gap-3 rounded-[3px] border bg-room-panel px-3 py-2 transition-colors duration-150 {pending
                    .playback.remember_volume
                    ? 'border-room-accent'
                    : 'border-room-line hover:border-room-line-strong'}"
                >
                  <input
                    type="checkbox"
                    checked={pending.playback.remember_volume}
                    disabled={saving}
                    onchange={(event) => setRememberVolume(event.currentTarget.checked)}
                    class="mt-1 accent-room-accent"
                  />
                  <span class="min-w-0 flex-1">
                    <span class="block text-[12.5px] text-room-text">Remember volume</span>
                    <span
                      class="mt-0.5 block font-mono text-[10.5px] leading-relaxed text-room-text-low"
                    >
                      Restore the last volume / mute setting when opening a new video.
                    </span>
                  </span>
                </label>
              </div>

              <div class="space-y-2 border-t border-room-line pt-4">
                <div class="font-mono text-[10px] uppercase tracking-[0.22em] text-room-text-low">
                  video chunk size
                </div>
                <p class="text-[11.5px] leading-relaxed text-room-text-mid">
                  How much of a video file Clowder fetches per range request. Larger values reduce
                  buffering pauses but use more memory and bandwidth per seek.
                </p>
                <div class="flex flex-wrap gap-1.5">
                  {#each chunkPresets as preset (preset.value)}
                    {@const isActive = pending.playback.video_chunk_mb === preset.value}
                    <button
                      type="button"
                      onclick={() => setVideoChunk(preset.value)}
                      disabled={saving}
                      class="h-7 rounded-[3px] border bg-room-panel px-3 font-mono text-[10.5px] uppercase tracking-[0.18em] transition-colors duration-150 disabled:opacity-50 {isActive
                        ? 'border-room-accent text-room-accent'
                        : 'border-room-line text-room-text-mid hover:border-room-line-strong hover:text-room-text'}"
                    >
                      {preset.label}
                    </button>
                  {/each}
                </div>
              </div>
            </div>
          {:else if activeSection === "appearance"}
            <div class="space-y-5 px-5 py-4">
              <div class="space-y-2">
                <div class="font-mono text-[10px] uppercase tracking-[0.22em] text-room-text-low">
                  theme
                </div>
                <div class="flex flex-wrap gap-1.5">
                  {#each themeOptions as option (option.value)}
                    {@const isActive = pending.appearance.theme === option.value}
                    <button
                      type="button"
                      onclick={() => setTheme(option.value)}
                      disabled={saving}
                      class="h-7 rounded-[3px] border bg-room-panel px-3 font-mono text-[10.5px] uppercase tracking-[0.18em] transition-colors duration-150 disabled:opacity-50 {isActive
                        ? 'border-room-accent text-room-accent'
                        : 'border-room-line text-room-text-mid hover:border-room-line-strong hover:text-room-text'}"
                    >
                      {option.label}
                    </button>
                  {/each}
                </div>
              </div>

              <div class="space-y-2 border-t border-room-line pt-4">
                <div class="font-mono text-[10px] uppercase tracking-[0.22em] text-room-text-low">
                  motion
                </div>
                <div class="space-y-1">
                  {#each motionOptions as option (option.value)}
                    {@const isActive = pending.appearance.motion === option.value}
                    <label
                      class="flex cursor-pointer items-start gap-3 rounded-[3px] border bg-room-panel px-3 py-2 transition-colors duration-150 {isActive
                        ? 'border-room-accent'
                        : 'border-room-line hover:border-room-line-strong'}"
                    >
                      <input
                        type="radio"
                        name="motion"
                        value={option.value}
                        checked={isActive}
                        disabled={saving}
                        onchange={() => setMotion(option.value)}
                        class="mt-1 accent-room-accent"
                      />
                      <span class="min-w-0 flex-1">
                        <span
                          class="block text-[12.5px] {isActive
                            ? 'text-room-accent'
                            : 'text-room-text'}"
                        >
                          {option.label}
                        </span>
                        <span
                          class="mt-0.5 block font-mono text-[10.5px] leading-relaxed text-room-text-low"
                        >
                          {option.description}
                        </span>
                      </span>
                    </label>
                  {/each}
                </div>
              </div>

              <div class="space-y-2 border-t border-room-line pt-4">
                <div class="font-mono text-[10px] uppercase tracking-[0.22em] text-room-text-low">
                  grid tile size
                </div>
                <div class="flex flex-wrap gap-1.5">
                  {#each tilePresets as preset (preset.value)}
                    {@const isActive = pending.appearance.grid_min_tile_px === preset.value}
                    <button
                      type="button"
                      onclick={() => setTilePx(preset.value)}
                      disabled={saving}
                      class="h-7 rounded-[3px] border bg-room-panel px-3 font-mono text-[10.5px] uppercase tracking-[0.18em] transition-colors duration-150 disabled:opacity-50 {isActive
                        ? 'border-room-accent text-room-accent'
                        : 'border-room-line text-room-text-mid hover:border-room-line-strong hover:text-room-text'}"
                    >
                      {preset.label}
                    </button>
                  {/each}
                </div>
                <p class="font-mono text-[10.5px] leading-relaxed text-room-text-low">
                  current minimum: {pending.appearance.grid_min_tile_px}px
                </p>
              </div>
            </div>
          {:else}
            <div class="space-y-4 px-5 py-4">
              <div>
                <div class="font-mono text-[10px] uppercase tracking-[0.22em] text-room-text-low">
                  version
                </div>
                <div class="mt-1 font-mono text-[13px] tabular-nums text-room-text">
                  {appVersion || "—"}
                </div>
              </div>
              <div>
                <div class="font-mono text-[10px] uppercase tracking-[0.22em] text-room-text-low">
                  source
                </div>
                <div class="mt-1 font-mono text-[12px] text-room-text-mid">
                  github.com/nyabi021/Clowder
                </div>
              </div>
              <div>
                <div class="font-mono text-[10px] uppercase tracking-[0.22em] text-room-text-low">
                  license
                </div>
                <div class="mt-1 font-mono text-[12px] text-room-text-mid">GPL-3.0-or-later</div>
              </div>
              <p class="pt-2 text-[11.5px] leading-relaxed text-room-text-low">
                Not affiliated with or endorsed by e621.
              </p>
            </div>
          {/if}
        </div>

        {#if activeSection !== "account" && activeSection !== "about"}
          <div
            class="flex items-center justify-between gap-2 border-t border-room-line px-5 py-3"
          >
            <span
              class="min-w-0 truncate font-mono text-[10.5px] {saveError
                ? 'text-room-fav'
                : saveStatus === 'saving'
                  ? 'text-room-text-mid'
                  : saveStatus === 'saved'
                    ? 'text-room-accent'
                    : 'text-room-text-low'}"
            >
              {saveError || saveStatus || (dirty ? "unsaved changes" : "")}
            </span>
            <div class="flex items-center gap-2">
              <button
                type="button"
                onclick={resetAll}
                disabled={!dirty || saving}
                class="h-8 rounded-[3px] border border-transparent px-3 font-mono text-[10.5px] uppercase tracking-[0.18em] text-room-text-mid transition-colors duration-150 hover:text-room-text disabled:opacity-40"
              >
                reset
              </button>
              <button
                type="button"
                onclick={saveAll}
                disabled={!dirty || saving}
                class="flex h-8 items-center gap-1.5 rounded-[3px] border border-room-accent bg-room-accent/10 px-3 font-mono text-[10.5px] uppercase tracking-[0.18em] text-room-accent transition-colors duration-150 hover:bg-room-accent/20 disabled:opacity-40"
              >
                {#if saving}
                  <span
                    class="size-2.5 animate-spin rounded-full border border-room-accent/40 border-t-room-accent"
                    aria-hidden="true"
                  ></span>
                {/if}
                save
              </button>
            </div>
          </div>
        {/if}
      </div>
    </div>
  </div>
</div>
