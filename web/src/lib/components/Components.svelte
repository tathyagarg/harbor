<script lang="ts">
  import { animate, eases } from "animejs";

  export let icon: string;
  export let title: string;
  export let description: string;

  export let loc: string;

  let card: HTMLDivElement;

  let intensity = 20;

  function handleMouseMove(e: MouseEvent) {
    const rect = card.getBoundingClientRect();

    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;
    const xPercentage = x / rect.width;
    const yPercentage = y / rect.height;
    const xRotation = xPercentage - 0.5;
    const yRotation = 0.5 - yPercentage;

    animate(card, {
      rotateX: yRotation * intensity,
      rotateY: xRotation * intensity,
      duration: 300,
      easing: eases.outQuad,
    });
  }

  function handleMouseLeave() {
    animate(card, {
      rotateX: 0,
      rotateY: 0,
      duration: 300,
      easing: eases.outQuad,
    });
  }
</script>

<div
  class="flex flex-col gap-4 p-6 rounded-lg shadow-lg border-1
  border-(white/20)"
  style="transform-style: preserve-3d;"
  bind:this={card}
  on:mousemove={handleMouseMove}
  on:mouseleave={handleMouseLeave}
  role="button"
  tabindex="0"
>
  <div class="flex justify-between">
    <div class="flex gap-2 items-center">
      <span class="text-2xl">{icon}</span>
      <h3 class="text-xl font-semibold">{title}</h3>
    </div>
    <p
      class="text-sm border-2
      border-white/20 px-4 py-1 rounded-full
      h-fit"
    >
      {loc} lines
    </p>
  </div>
  <p class="text-sm">{description}</p>
</div>
