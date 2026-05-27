<script lang="ts">
  type Props = {
    group: string;
    tags: readonly string[];
    onTagClick?: (tag: string) => void;
    maxVisible?: number;
    padding?: "tight" | "comfortable";
  };

  let { group, tags, onTagClick, maxVisible, padding = "comfortable" }: Props = $props();

  let visibleTags = $derived(
    maxVisible !== undefined && tags.length > maxVisible ? tags.slice(0, maxVisible) : tags,
  );
  let overflowCount = $derived(
    maxVisible !== undefined && tags.length > maxVisible ? tags.length - maxVisible : 0,
  );
  let sectionPadding = $derived(padding === "tight" ? "px-4 py-3" : "px-5 py-4");
</script>

{#if tags.length > 0}
  <section class="border-b border-room-line {sectionPadding}">
    <div class="mb-2 flex items-baseline justify-between">
      <div class="font-mono text-[10px] uppercase tracking-[0.22em] text-room-text-low">
        {group}
      </div>
      <div class="font-mono text-[10px] tabular-nums text-room-text-low">
        {tags.length}
      </div>
    </div>
    <div class="flex flex-wrap gap-1">
      {#each visibleTags as tag (tag)}
        {#if onTagClick}
          <button
            type="button"
            onclick={() => onTagClick?.(tag)}
            class="inline-flex h-6 max-w-full items-center truncate rounded-[2px] border border-room-line bg-room-panel px-2 font-mono text-[10.5px] text-room-text-mid transition-colors duration-150 hover:border-room-accent hover:text-room-accent"
          >
            {tag}
          </button>
        {:else}
          <span
            class="inline-flex h-6 max-w-full items-center truncate rounded-[2px] border border-room-line bg-room-panel/60 px-2 font-mono text-[10.5px] text-room-text-mid"
          >
            {tag}
          </span>
        {/if}
      {/each}
      {#if overflowCount > 0}
        <span class="inline-flex h-6 items-center px-1 font-mono text-[10.5px] text-room-text-low">
          +{overflowCount} more
        </span>
      {/if}
    </div>
  </section>
{/if}
