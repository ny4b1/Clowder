<script lang="ts">
  import type { TagSuggestion } from "../lib/types";
  import { fetchTagSuggestions } from "../lib/e621";
  import {
    applySuggestionToQuery,
    categoryLabel,
    currentToken,
    localMetatagSuggestions,
    looksLikeMetatag,
  } from "../lib/search";

  type Props = {
    query: string;
    status: string;
    loading: boolean;
    username: string | null;
    onQueryChange: (query: string) => void;
    onSearch: () => void;
    onOpenAccount: () => void;
  };

  let { query, status, loading, username, onQueryChange, onSearch, onOpenAccount }: Props =
    $props();

  let suggestions = $state<TagSuggestion[]>([]);
  let showSuggestions = $state(false);
  let activeSuggestion = $state(0);
  let autocompleteTimer: number | undefined;

  function updateQuery(value: string) {
    onQueryChange(value);
    window.clearTimeout(autocompleteTimer);
    const token = currentToken(value);
    const metatagSuggestions = localMetatagSuggestions(token.raw);
    if (metatagSuggestions.length > 0) {
      suggestions = metatagSuggestions;
      activeSuggestion = 0;
      showSuggestions = true;
      return;
    }

    if (token.search.length < 2 || token.search.includes(":") || looksLikeMetatag(token.search)) {
      suggestions = [];
      showSuggestions = false;
      return;
    }

    autocompleteTimer = window.setTimeout(() => {
      void loadSuggestions(token.search);
    }, 160);
  }

  async function loadSuggestions(term: string) {
    try {
      const result = await fetchTagSuggestions(term);
      suggestions = result;
      activeSuggestion = 0;
      showSuggestions = result.length > 0;
    } catch {
      suggestions = [];
      showSuggestions = false;
    }
  }

  function onKeydown(event: KeyboardEvent) {
    if (!showSuggestions || suggestions.length === 0) return;

    if (event.key === "ArrowDown") {
      event.preventDefault();
      activeSuggestion = (activeSuggestion + 1) % suggestions.length;
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      activeSuggestion = (activeSuggestion - 1 + suggestions.length) % suggestions.length;
    } else if (event.key === "Tab" || event.key === "Enter") {
      event.preventDefault();
      applySuggestion(suggestions[activeSuggestion]);
    } else if (event.key === "Escape") {
      showSuggestions = false;
    }
  }

  function applySuggestion(suggestion: TagSuggestion) {
    onQueryChange(applySuggestionToQuery(query, suggestion));
    suggestions = [];
    showSuggestions = false;
  }
</script>

<header class="flex items-center gap-4 border-b border-room-line px-4">
  <div class="flex shrink-0 select-none items-baseline gap-2">
    <span class="font-mono text-[15px] font-medium leading-none tracking-tight text-room-text">
      Clowder
    </span>
    <span class="font-mono text-[10px] uppercase leading-none tracking-[0.22em] text-room-text-low">
      browser
    </span>
  </div>

  <div class="h-5 w-px bg-room-line"></div>

  <form
    class="relative w-full max-w-[640px]"
    onsubmit={(event) => {
      event.preventDefault();
      onSearch();
    }}
  >
    <div
      class="flex h-8 items-stretch rounded-[3px] border border-room-line bg-room-panel transition-colors duration-150 focus-within:border-room-accent"
    >
      <div class="flex items-center pl-2.5 pr-2 text-room-text-low">
        <svg
          class="size-3.5"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="1.75"
          stroke-linecap="round"
          stroke-linejoin="round"
          aria-hidden="true"
        >
          <circle cx="11" cy="11" r="7" />
          <path d="m21 21-4.3-4.3" />
        </svg>
      </div>
      <input
        value={query}
        autocomplete="off"
        spellcheck="false"
        oninput={(event) => updateQuery(event.currentTarget.value)}
        onkeydown={onKeydown}
        onfocus={() => updateQuery(query)}
        onblur={() => window.setTimeout(() => (showSuggestions = false), 120)}
        placeholder="search tags"
        class="min-w-0 flex-1 bg-transparent pr-2 text-[12.5px] text-room-text outline-none placeholder:text-room-text-low"
      />
      <button
        type="submit"
        disabled={loading}
        class="flex items-center gap-1.5 border-l border-room-line px-3.5 font-mono text-[10.5px] uppercase tracking-[0.18em] text-room-text-mid transition-colors duration-150 hover:bg-room-panel-hi hover:text-room-text disabled:opacity-50"
      >
        {#if loading}
          <span
            class="size-2.5 animate-spin rounded-full border border-room-line-strong border-t-room-accent"
            aria-hidden="true"
          ></span>
        {/if}
        search
      </button>
    </div>

    {#if showSuggestions}
      <div
        class="absolute left-0 right-0 top-9 z-30 max-h-72 overflow-auto rounded-[3px] border border-room-line bg-room-panel-hi shadow-[0_8px_24px_rgba(0,0,0,0.45)]"
        role="listbox"
      >
        {#each suggestions as suggestion, index (suggestion.id)}
          <button
            type="button"
            class:active={index === activeSuggestion}
            class="grid w-full grid-cols-[1fr_auto] items-center gap-3 px-3 py-1.5 text-left text-[12.5px] text-room-text transition-colors duration-100 hover:bg-room-bg/60 [&.active]:bg-room-bg [&.active]:text-room-accent"
            onmousedown={(event) => {
              event.preventDefault();
              applySuggestion(suggestion);
            }}
          >
            <span class="truncate">{suggestion.name}</span>
            <span class="font-mono text-[10px] tabular-nums text-room-text-low">
              <span class="text-room-text-mid">{categoryLabel(suggestion.category)}</span>
              {#if suggestion.post_count > 0}
                <span class="px-1 text-room-line-strong">·</span>
                {suggestion.post_count.toLocaleString()}
              {/if}
            </span>
          </button>
        {/each}
      </div>
    {/if}
  </form>

  <div class="ml-auto flex items-center gap-3 font-mono text-[10.5px] tabular-nums text-room-text-mid">
    <span class="flex items-center gap-2">
      <span
        class="size-1.5 rounded-full {loading
          ? 'animate-pulse bg-room-accent'
          : 'bg-room-line-strong'}"
        aria-hidden="true"
      ></span>
      {status}
    </span>
    <span class="h-4 w-px bg-room-line"></span>
    <button
      type="button"
      onclick={onOpenAccount}
      class="flex h-7 items-center gap-1.5 rounded-[3px] border border-room-line bg-room-panel px-2.5 text-[10.5px] uppercase tracking-[0.18em] text-room-text-mid transition-colors duration-150 hover:border-room-line-strong hover:text-room-text"
    >
      <svg
        class="size-3"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="1.75"
        stroke-linecap="round"
        stroke-linejoin="round"
        aria-hidden="true"
      >
        <circle cx="12" cy="8" r="4" />
        <path d="M4 21a8 8 0 0 1 16 0" />
      </svg>
      {username ?? "sign in"}
    </button>
  </div>
</header>
