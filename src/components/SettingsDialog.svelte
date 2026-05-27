<script lang="ts">
  import { onMount, untrack } from "svelte";
  import CloseIcon from "./icons/CloseIcon.svelte";
  import Spinner from "./icons/Spinner.svelte";
  import AboutSection from "./settings/AboutSection.svelte";
  import AccountSection from "./settings/AccountSection.svelte";
  import AppearanceSection from "./settings/AppearanceSection.svelte";
  import DownloadsSection from "./settings/DownloadsSection.svelte";
  import PlaybackSection from "./settings/PlaybackSection.svelte";
  import { errMsg } from "../lib/errors";
  import { settingsStore } from "../lib/settings-store.svelte";
  import type { Settings, SettingsSection } from "../lib/types";

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

  onMount(() => {
    if (!settingsStore.loaded) {
      void settingsStore.load().then(() => {
        pending = structuredClone($state.snapshot(settingsStore.current));
      });
    } else {
      pending = structuredClone($state.snapshot(settingsStore.current));
    }
  });

  let dirty = $derived.by(
    () => JSON.stringify(pending) !== JSON.stringify(settingsStore.current),
  );

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
      saveError = errMsg(error);
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
    { id: "downloads", label: "Downloads" },
    { id: "playback", label: "Playback" },
    { id: "appearance", label: "Appearance" },
    { id: "about", label: "About" },
  ];

  let showSaveBar = $derived(activeSection !== "account" && activeSection !== "about");

  // dirty takes precedence so "saved" doesn't linger after the user edits again.
  let statusText = $derived(saveError || (dirty ? "unsaved changes" : saveStatus));
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
        disabled={accountSaving || saving}
        class="flex size-7 items-center justify-center rounded-[3px] border border-room-line bg-room-panel text-room-text-mid transition-colors duration-150 hover:border-room-line-strong hover:text-room-text disabled:opacity-50"
        aria-label="Close"
      >
        <CloseIcon />
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
            <AccountSection
              {username}
              {usernameInput}
              {apiKeyInput}
              {accountStatus}
              {accountSaving}
              {onSubmit}
              {onSignOut}
              {onUsernameInput}
              {onApiKeyInput}
            />
          {:else if activeSection === "downloads"}
            <DownloadsSection
              bind:downloads={pending.downloads}
              {saving}
              onError={(message) => (saveError = message)}
            />
          {:else if activeSection === "playback"}
            <PlaybackSection bind:playback={pending.playback} {saving} />
          {:else if activeSection === "appearance"}
            <AppearanceSection bind:appearance={pending.appearance} {saving} />
          {:else}
            <AboutSection />
          {/if}
        </div>

        {#if showSaveBar}
          <div class="flex items-center justify-between gap-2 border-t border-room-line px-5 py-3">
            <span
              class="min-w-0 truncate font-mono text-[10.5px] {saveError
                ? 'text-room-fav'
                : saveStatus === 'saving'
                  ? 'text-room-text-mid'
                  : saveStatus === 'saved' && !dirty
                    ? 'text-room-accent'
                    : 'text-room-text-low'}"
            >
              {statusText}
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
                  <Spinner class="size-2.5 border border-room-accent/40 border-t-room-accent" />
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
