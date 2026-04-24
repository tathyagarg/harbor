<script>
  import Heading from "$lib/components/Heading.svelte";
  import Icon from "@iconify/svelte";
  import { animate } from "animejs";
  import { onMount } from "svelte";

  onMount(async () => {
    animate("#demo-ss", {
      opacity: 0,
      translateX: -100,
      duration: 0,
    });

    animate("#gear", {
      rotate: "1.5turn",
      duration: 3000,
      ease: "linear",
    });

    await animate("#gear", {
      opacity: 0,
      scale: 0.5,
      duration: 1000,
      delay: 2000,
    });

    animate("#demo-ss", {
      opacity: 1,
      translateX: 0,
      duration: 1000,
    });
  });
</script>

<div class="w-full h-full p-2 relative">
  <Icon
    icon="mdi:settings"
    class="w-64 h-64 text-emphasis-1 absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2"
    id="gear"
  />

  <div
    id="demo-ss"
    class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2"
  >
    <div class="text-emphasis-1/25 border-b-1 border-emphasis-1/25 mb-4">
      <Heading text="Final Result" />
    </div>
    <img src="/demo.png" alt="Demo Screenshot" class="h-72" />
  </div>
</div>
