<script lang="ts">
  import Step from "$lib/components/Step.svelte";
  import HTML from "$lib/components/steps/HTML.svelte";
  import HTTP from "$lib/components/steps/HTTP.svelte";
  import TTFLoader from "$lib/components/steps/TTFLoader.svelte";

  let panel: HTMLDivElement;

  const steps = [
    "TTF Loader",
    "HTTP Client",
    "HTML Parser -> DOM",
    "Link Resolution",
    "CSS Parser -> CSSOM",
    "Style Resolution & Cascading",
    "Layout Engine (Box Model)",
    "TTF Rasterizer",
    "Painting",
  ];

  let selected_step = $state(2);

  import { animate, onScroll, stagger } from "animejs";
  import { onMount } from "svelte";

  onMount(() => {
    const elems = document.querySelectorAll(".animate");

    animate(elems, {
      opacity: [0, 1],
      translateX: [-20, 0],
      scale: [0.85, 1],
      filter: ["blur(10px)", "blur(0)"],
      duration: 300,
      delay: stagger(100),
      ease: "cubicBezier(0.25, 0.1, 0.25, 1)",
      autoplay: onScroll({
        target: "#header",
        debug: false,
      }),
    });
  });

  async function changeStep(i: number) {
    if (i === selected_step) return;

    await animate(panel, {
      opacity: [1, 0],
      translateX: [0, -40],
      filter: ["blur(0)", "blur(5px)"],
      duration: 150,
      easing: "cubicBezier(0.25, 0.1, 0.25, 1)",
    });

    selected_step = i;

    await animate(panel, {
      opacity: [0, 1],
      translateX: [40, 0],
      filter: ["blur(5px)", "blur(0)"],
      duration: 150,
      easing: "cubicBezier(0.25, 0.1, 0.25, 1)",
    });
  }
</script>

<div class="py-4 flex flex-row gap-8">
  <ol class="list-decimal pl-4 flex-1">
    {#each steps as step, i}
      <Step
        title={step}
        onClick={() => {
          changeStep(i);
        }}
        last={i === steps.length - 1}
      />
    {/each}
  </ol>

  <div
    bind:this={panel}
    class="flex-2 bg-(--cerulean)/25 rounded-lg relative animate"
  >
    <span class="absolute top-2 left-2"
      >Step {selected_step + 1}: <b>{steps[selected_step]}</b></span
    >

    {#if selected_step === 0}
      <TTFLoader />
    {:else if selected_step === 1}
      <HTTP />
    {:else if selected_step == 2}
      <HTML />
    {/if}
  </div>
</div>
