<!-- svelte-ignore state_referenced_locally -->

<script lang="ts">
  import { GITHUB_URL } from "$lib";
  import HeroTag from "$lib/components/HeroTag.svelte";
  import Stat from "$lib/components/Stat.svelte";
  import { animate, createTimeline, stagger } from "animejs";
  import { onMount } from "svelte";
  import type { PageProps } from "./$types";

  let { data }: PageProps = $props();

  let stats = [
    {
      value: Math.floor(data.lines.total / 1000),
      title: "Lines of Code",
      suffix: "K+",
    },
    {
      value: Math.floor(data.file_count / 10) * 10,
      title: "Files",
      suffix: "+",
    },
    { value: 6, title: "Core Modules", suffix: null },
    { value: 0, title: "External Dependencies", suffix: null },
  ];

  let timeline = createTimeline({
    autoplay: true,
    defaults: {
      ease: "inOutQuad",
      duration: 500,
    },
  });

  onMount(() => {
    animate(".anim-2", {
      opacity: 0,
      translateY: 50,
      duration: 0,
    });

    animate(".anim-3", {
      opacity: 0,
      translateX: -20,
      duration: 0,
    });

    timeline
      .add(".anim", {
        opacity: [0, 1],
        translateY: [20, 0],
      })
      .add(".anim-2", {
        opacity: [0, 1],
        translateY: [50, 0],
        delay: 200,
      })
      .add(".anim-3", {
        opacity: [0, 1],
        translateX: [-20, 0],
        delay: stagger(100),
      });
  });
</script>

<div class="w-[50%] mx-auto">
  <div class="h-screen flex items-center justify-center flex-col">
    <h1 class="text-8xl anim">Welcome to</h1>
    <h1 class="text-8xl text-emphasis-2 anim">Harbor Browser</h1>
    <div class="flex mb-6">
      <HeroTag
        href={GITHUB_URL}
        icon="mdi:github"
        color="var(--color-gh-green)"
        text="View Source"
      />

      <HeroTag
        href=""
        icon="mdi:hammer"
        color="var(--color-scratch-red)"
        text="Built from Scratch"
      />
    </div>

    <p class="text-subtext text-center anim">
      Harbor Browser is a custom web browser engine built from scratch with Rust
      and Zig. <span class="text-text"
        >Every core system - from networking to parsing to layouts to rendering,
        and more - was implemented manually without relying on existing browser
        engines or frameworks</span
      >
    </p>

    <div class="grid grid-cols-4 gap-12 mt-12 anim-2">
      {#each stats as stat}
        <Stat {...stat} />
      {/each}
    </div>
  </div>

  <div class="h-screen">
    <div class="h-[80%] py-[10%]">
      <div class="flex gap-4 items-baseline my-2">
        <span class="text-emphasis-1">01</span>
        <div class="w-full bg-emphasis-1 h-0.5"></div>
      </div>
      <h1 class="text-emphasis-2 text-6xl my-4">The Pipeline</h1>
      <p class="text-subtext">
        Harbor Browser's architecture is divided into several core systems that
        work together to process and render web content.
      </p>

      <div></div>
    </div>
  </div>
</div>
