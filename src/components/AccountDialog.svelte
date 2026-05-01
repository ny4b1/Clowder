<script lang="ts">
  type Props = {
    username: string | null;
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
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="fixed inset-0 z-50 flex items-center justify-center bg-black/55 backdrop-blur-[2px]"
  onclick={onBackdropClick}
>
  <div
    class="w-[380px] rounded-[4px] border border-room-line-strong bg-room-panel-hi shadow-[0_24px_72px_rgba(0,0,0,0.6)]"
    role="dialog"
    aria-modal="true"
    aria-label="Account"
  >
    <form onsubmit={onSubmit}>
      <div class="flex items-center justify-between border-b border-room-line px-5 py-3">
        <div>
          <div class="font-mono text-[10px] uppercase tracking-[0.22em] text-room-text-low">
            account
          </div>
          <div class="mt-0.5 text-[13px] text-room-text">
            {username ? "signed in" : "e621 credentials"}
          </div>
        </div>
        <button
          type="button"
          onclick={onClose}
          disabled={accountSaving}
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

      <div class="space-y-3 px-5 py-4">
        <label class="block">
          <span class="mb-1 block font-mono text-[10px] uppercase tracking-[0.22em] text-room-text-low">
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
          <span class="mb-1 block font-mono text-[10px] uppercase tracking-[0.22em] text-room-text-low">
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
          generate at e621.net/users/home -> manage api access. stored in the macos keychain.
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
      </div>

      <div class="flex items-center justify-between gap-2 border-t border-room-line px-5 py-3">
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
        <div class="flex items-center gap-2">
          <button
            type="button"
            onclick={onClose}
            disabled={accountSaving}
            class="h-8 rounded-[3px] border border-transparent px-3 font-mono text-[10.5px] uppercase tracking-[0.18em] text-room-text-mid transition-colors duration-150 hover:text-room-text disabled:opacity-50"
          >
            cancel
          </button>
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
  </div>
</div>
