<script lang="ts">
  import type { Preset, SortMode } from "../lib/types";

  type Props = {
    presets: Preset[];
    activePreset: string | null;
    sortMode: SortMode;
    onApplyPreset: (preset: Preset) => void;
    onApplySort: (sort: SortMode) => void;
  };

  const sortOptions: { label: string; value: SortMode }[] = [
    { label: "Latest", value: "latest" },
    { label: "Popular", value: "popular" },
  ];

  let { presets, activePreset, sortMode, onApplyPreset, onApplySort }: Props = $props();
</script>

<nav class="flex items-stretch border-b border-room-line bg-room-panel/50">
  <div
    class="flex shrink-0 items-center border-r border-room-line px-4 font-mono text-[10px] uppercase tracking-[0.22em] text-room-text-low"
  >
    presets
  </div>
  {#each presets as preset (preset.value || preset.label)}
    {@const isActive = activePreset === preset.value}
    <button
      type="button"
      onclick={() => onApplyPreset(preset)}
      class="relative px-4 text-[12px] tracking-tight transition-colors duration-150 hover:bg-room-panel-hi hover:text-room-text {isActive
        ? 'bg-room-panel-hi text-room-accent'
        : 'text-room-text-mid'}"
    >
      {preset.label}
      {#if isActive}
        <span class="absolute inset-x-3 bottom-0 h-px bg-room-accent" aria-hidden="true"></span>
      {/if}
    </button>
  {/each}
  <div class="ml-auto flex items-center border-l border-room-line">
    <div
      class="flex shrink-0 items-center px-4 font-mono text-[10px] uppercase tracking-[0.22em] text-room-text-low"
    >
      sort
    </div>
    {#each sortOptions as option (option.value)}
      {@const isActive = !activePreset && sortMode === option.value}
      <button
        type="button"
        onclick={() => onApplySort(option.value)}
        class="relative px-4 text-[12px] tracking-tight transition-colors duration-150 hover:bg-room-panel-hi hover:text-room-text {isActive
          ? 'bg-room-panel-hi text-room-accent'
          : 'text-room-text-mid'}"
      >
        {option.label}
        {#if isActive}
          <span class="absolute inset-x-3 bottom-0 h-px bg-room-accent" aria-hidden="true"></span>
        {/if}
      </button>
    {/each}
  </div>
</nav>
