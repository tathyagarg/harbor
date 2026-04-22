<script lang="ts">
  let { title, hasDots = false, children } = $props();
</script>

<div
  class="rounded-lg border-1 border-emphasis-1/20 overflow-hidden w-full h-full"
>
  <div
    class="flex items-center gap-2 p-2 bg-emphasis-1/10 border-b-1 border-emphasis-1/20"
  >
    {#if hasDots}
      <div class="w-[8px] h-[8px] rounded-full bg-scratch-red"></div>
      <div class="w-[8px] h-[8px] rounded-full bg-[#f0c040]"></div>
      <div class="w-[8px] h-[8px] rounded-full bg-gh-green"></div>
    {/if}
    <span class="text-[12px] text-emphasis-1/75 font-mono uppercase">
      {title}
    </span>
  </div>
  <div class="h-full bg-black/20 p-2 relative">
    {@render children?.()}
  </div>
</div>
