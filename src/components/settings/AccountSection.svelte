<script lang="ts">
  import Spinner from "../icons/Spinner.svelte";
  import { siteLabels, type Site } from "../../lib/site";

  type Props = {
    site: Site;
    username: string | null;
    usernameInput: string;
    apiKeyInput: string;
    accountStatus: string;
    accountSaving: boolean;
    onSubmit: (event: Event) => void;
    onSignOut: () => void;
    onUsernameInput: (value: string) => void;
    onApiKeyInput: (value: string) => void;
  };

  let {
    site,
    username,
    usernameInput,
    apiKeyInput,
    accountStatus,
    accountSaving,
    onSubmit,
    onSignOut,
    onUsernameInput,
    onApiKeyInput,
  }: Props = $props();
</script>

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
      {siteLabels[site]} account · generate at {site}.net/users/home → manage api access. stored in
      the system keychain.
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
          <Spinner class="size-2.5 border border-room-accent/40 border-t-room-accent" />
        {/if}
        {username ? "update" : "sign in"}
      </button>
    </div>
  </div>
</form>
