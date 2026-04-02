<script lang="ts">
  import { GITHUB_URL } from "$lib";
  import HeroTag from "$lib/components/HeroTag.svelte";
  import Stat from "$lib/components/Stat.svelte";
  import { animate, createTimeline, stagger } from "animejs";
  import { onMount } from "svelte";

  let stats = [
    { value: 53, title: "Lines of Code", suffix: "K+" },
    { value: 100, title: "Files", suffix: "+" },
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
      .label("start")
      .add(
        ".anim",
        {
          opacity: [0, 1],
          translateY: [20, 0],
        },
        "start",
      )
      .add(
        ".anim-2",
        {
          opacity: [0, 1],
          translateY: [50, 0],
        },
        "start+=200",
      )
      .add(
        ".anim-3",
        {
          opacity: [0, 1],
          translateX: [-20, 0],
          delay: stagger(100),
        },
        "start+=500",
      );
  });
</script>

<div class="h-screen w-screen flex items-center justify-center flex-col">
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

  <p class="text-subtext w-[50%] text-center leading-[2] anim">
    Harbor Browser is a custom web browser engine built from scratch with Rust
    and Zig. <span class="text-text"
      >Every core system - from networking to parsing to layouts to rendering,
      and more - was implemented manually without relying on existing browser
      engines or frameworks</span
    >
  </p>

  <div class="w-[50%] grid grid-cols-4 gap-12 mt-12 anim-2">
    {#each stats as stat}
      <Stat {...stat} />
    {/each}
  </div>
</div>
