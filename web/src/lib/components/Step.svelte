<script lang="ts">
  import { animate, eases } from "animejs";

  export let title: string;
  export let onClick: () => void;

  export let last: boolean = false;

  let step: HTMLLIElement;

  let intensity = 20;

  function handleMouseMove(e: MouseEvent) {
    const rect = step.getBoundingClientRect();

    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;
    const xPercentage = x / rect.width;
    const yPercentage = y / rect.height;
    const xRotation = xPercentage - 0.5;
    const yRotation = 0.5 - yPercentage;

    animate(step, {
      rotateX: yRotation * intensity,
      rotateY: xRotation * intensity,
      duration: 300,
      easing: eases.outQuad,
    });
  }

  function handleMouseLeave() {
    animate(step, {
      rotateX: 0,
      rotateY: 0,
      duration: 300,
      easing: eases.outQuad,
    });
  }
</script>

<li class="animate min-w-full" bind:this={step}>
  <button
    on:click={onClick}
    class="bg-(--deep-space-blue) py-2 border-1 border-(--celadon)
    rounded-lg relative cursor-pointer hover:bg-(--cerulean)/75
    transition-colors duration-200 text-center w-full h-full"
    type="button"
    tabindex="0"
    class:mb-4={!last}
    on:mousemove={handleMouseMove}
    on:mouseleave={handleMouseLeave}
  >
    {title}
  </button>
</li>
