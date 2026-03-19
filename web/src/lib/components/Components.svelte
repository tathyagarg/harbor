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
  class="flex flex-col gap-4 bg-deep-space-blue p-6 rounded-lg shadow-lg border-1
  border-celadon"
  style="transform-style: preserve-3d;"
  bind:this={card}
  on:mousemove={handleMouseMove}
  on:mouseleave={handleMouseLeave}
  role="button"
  tabindex="0"
>
  <div class="flex justify-between">
    <span class="text-2xl text-celadon">{icon}</span>
    <p
      class="text-(white/50) text-sm border-2
      border-white/50 px-4 py-1 rounded-full
      h-fit"
    >
      ~{loc} lines
    </p>
  </div>
  <h3 class="text-xl font-semibold">{title}</h3>
  <p class="text-sm">{description}</p>
</div>
