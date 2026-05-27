<script lang="ts">
  type Props = {
    class?: string;
  };

  let { class: className = "size-3.5" }: Props = $props();
</script>

<svg
  class={className}
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
