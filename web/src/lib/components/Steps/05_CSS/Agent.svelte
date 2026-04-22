<script lang="ts">
  import { animate } from "animejs";
  import { onMount } from "svelte";

  let { url } = $props();
  let done = $state(false);

  onMount(() => {
    animate(".progress", {
      width: ["0", "100%"],
      duration: 2000,
      delay: 2000,
      ease: "outCirc",
    });

    setTimeout(() => {
      done = true;
    }, 4000);
  });
</script>

<div
  class={`agent opacity-0 w-full px-2 py-1 rounded-lg mb-1 border-1 ${done ? "bg-emphasis-3/20" : "bg-emphasis-1/20"}`}
  class:border-emphasis-3={done}
  class:border-emphasis-1={!done}
>
  <div class="flex justify-between">
    <span class="text-[12px] font-mono text-emphasis-1 rounded-lg mb-1">
      {url}
    </span>
  </div>
  <div class="progress bg-emphasis-3 h-1 rounded mb-1"></div>
</div>
