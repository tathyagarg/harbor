<script lang="ts">
  let { url, state } = $props();
</script>

<div
  class="w-full text-[12px] font-mono text-emphasis-1/80 px-2 py-1 rounded-lg mb-1
  opacity-0 exurl flex justify-between gap-1 border-1 border-pretty-blue bg-pretty-blue/20"
>
  <span>
    {url}
  </span>
  <span class="text-emphasis-3">
    {state}
  </span>
</div>
