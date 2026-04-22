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
  class={`agent grid grid-cols-4 gap-2 opacity-0 w-full px-2 py-1 rounded-lg mb-1 border-1 ${done ? "bg-emphasis-3/20" : "bg-emphasis-1/20"}`}
  class:border-emphasis-3={done}
  class:border-emphasis-1={!done}
>
  <div class="flex justify-between overflow-hidden">
    <span
      class="text-xs font-mono text-emphasis-1 rounded-lg mb-1 text-ellipsis whitespace-nowrap"
    >
      {url}
    </span>
  </div>
  <div class="w-full h-full col-span-3 flex items-center">
    <div class="progress col-span-3 bg-emphasis-3 h-1 rounded"></div>
  </div>
</div>
