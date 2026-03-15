<script lang="ts">
  import Icon from "@iconify/svelte";
  import { animate, eases } from "animejs";

  export let selected: boolean = false;
  export let title: string;
  export let onClick: () => void;

  export let last: boolean = false;

  export let icon: string;

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
    class:bg-(--cerulean)={selected}
    class:bg-(--deep-space-blue)={!selected}
    class="py-2 border-1 border-(--celadon)
    rounded-lg relative cursor-pointer hover:bg-(--cerulean)/75
    transition-colors duration-200 text-center w-full h-full text-2xl font-bold"
    type="button"
    tabindex="0"
    class:mb-4={!last}
    on:mousemove={handleMouseMove}
    on:mouseleave={handleMouseLeave}
  >
    <Icon {icon} class="inline-block" />
    {title}
  </button>
</li>
