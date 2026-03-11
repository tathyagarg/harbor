<script lang="ts">
  import Step from "$lib/components/Step.svelte";
  import Cascade from "$lib/components/steps/Cascade.svelte";
  import CSS from "$lib/components/steps/CSS.svelte";
  import HTML from "$lib/components/steps/HTML.svelte";
  import HTTP from "$lib/components/steps/HTTP.svelte";
  import Layout from "$lib/components/steps/Layout.svelte";
  import Links from "$lib/components/steps/Links.svelte";
  import Paint from "$lib/components/steps/Paint.svelte";
  import Rasterize from "$lib/components/steps/Rasterize.svelte";
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

  let selected_step = $state(0);

  import { animate, cubicBezier, onScroll, stagger } from "animejs";
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
      ease: cubicBezier(0.25, 0.1, 0.25, 1),
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

<h1 class="text-8xl font-bold text-center">Render Pipeline</h1>

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
    {:else if selected_step == 3}
      <Links />
    {:else if selected_step == 4}
      <CSS />
    {:else if selected_step == 5}
      <Cascade />
    {:else if selected_step == 6}
      <Layout />
    {:else if selected_step == 7}
      <Rasterize />
    {:else}
      <Paint />
    {/if}
  </div>
</div>

<div class="bg-(--celadon) text-black p-4 rounded-lg my-8 animate">
  <h1 class="text-4xl font-bold">A confession</h1>
  <hr />

  <p class="text-lg mt-4 leading-relaxed">
    The functioning of Harbor Browser is far too complex to be accurately
    represented by a single step-by-step diagram - it's impossible to summarize
    30 thousand lines of code comprising deeply interlinked modules and services
    in just a few steps. The steps outlined above are a very high-level overview
    of the render pipeline, and do not capture the full complexity and nuance of
    how Harbor Browser actually works.
  </p>
</div>
