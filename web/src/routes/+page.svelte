<!-- svelte-ignore state_referenced_locally -->

<script lang="ts">
  import { GITHUB_URL } from "$lib";
  import HeroTag from "$lib/components/HeroTag.svelte";
  import Stat from "$lib/components/Stat.svelte";
  import { animate, createTimeline, onScroll, stagger } from "animejs";
  import { onMount } from "svelte";
  import type { PageProps } from "./$types";
  import Font from "$lib/components/Steps/Font/Step.svelte";
  import HTTP from "$lib/components/Steps/HTTP.svelte";

  let { data }: PageProps = $props();
  let playStep = $state(false);

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
        "start+=500",
      )
      .add(
        ".anim-3",
        {
          opacity: [0, 1],
          translateX: [-20, 0],
          delay: stagger(100),
        },
        "start+=1000",
      );

    animate("#pipeline", {
      opacity: 0,
      translateY: 50,
      duration: 0,
    });

    animate("#pipeline", {
      opacity: [0, 1],
      translateY: [50, 0],
      delay: 1000,
      duration: 500,
      ease: "inOutQuad",
      autoplay: onScroll({
        container: document.getElementsByName("html")[0],
      }),
    });
  });

  const steps = [
    {
      short: "Font",
      title: "Font Reader",
      description:
        "Reads TrueType fonts and parses them into tables like <code>cmap</code>, <code>glyf</code>, etc.",
    },
    { short: "HTTP", title: "HTTP Client" },
    { short: "HTML", title: "HTML Parser" },
    { short: "Links", title: "Link Resolver" },
    { short: "CSS", title: "CSS Parser" },
    { short: "Cascade", title: "Cascade & Inheritance" },
    { short: "Layout", title: "Layout Engine" },
    { short: "Rasterize", title: "Rasterizer" },
    { short: "Paint", title: "Paint Engine" },
  ];

  const radius = 25;

  let selected_step = $state(0);

  async function switchTo(n: number) {
    await animate("#inner-step", {
      opacity: [1, 0],
      translateX: [0, -200],
      duration: 250,
      easing: "easeInOutQuad",
    });

    animate("#inner-step", {
      translateX: 200,
      duration: 0,
    });

    selected_step = n;

    await animate("#inner-step", {
      opacity: [0, 1],
      translateX: [200, 0],
      duration: 250,
      easing: "easeInOutQuad",
    });
  }
</script>

<div class="w-[50%] mx-auto" id="page">
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

  <div class="h-screen" id="pipeline">
    <div class="h-full py-[10%] flex flex-col">
      <div class="flex gap-4 items-baseline my-2">
        <span class="text-emphasis-1">01</span>
        <div class="w-full bg-emphasis-1 h-0.5"></div>
      </div>
      <h1 class="text-emphasis-2 text-6xl my-4">The Pipeline</h1>
      <p class="text-subtext">
        Harbor Browser's architecture is divided into several core systems that
        work together to process and render web content.
      </p>

      <svg class="mx-auto" viewBox="0 0 900 150">
        <defs>
          <filter id="shadow" x="-50%" y="-50%" width="200%" height="200%">
            <feDropShadow
              dx="0"
              dy="0"
              stdDeviation="10"
              flood-opacity="0.5"
              flood-color="var(--color-emphasis-2)"
            />
          </filter>
        </defs>

        {#each steps as step, i}
          {@const color =
            i <= selected_step
              ? "var(--color-emphasis-2)"
              : "var(--color-emphasis-1)"}
          <g
            transform={`translate(${i * 4 * radius}, 75)`}
            onclick={() => switchTo(i)}
            onkeydown={(e) => {
              if (e.key === "Enter" || e.key === " ") {
                switchTo(i);
              }
            }}
            style="cursor: pointer"
            role="button"
            aria-pressed={i === selected_step}
            tabindex="0"
          >
            <circle
              cx={2 * radius}
              cy="0"
              r={radius}
              fill={`rgba(from ${color} r g b / 25%)`}
              stroke={color}
              stroke-width="2"
              filter={i === selected_step ? "url(#shadow)" : "none"}
            />
            <text
              x={2 * radius}
              y="1.8"
              text-anchor="middle"
              dominant-baseline="middle"
              fill={color}
              font-size="18"
              font-family="monospace"
            >
              {i}
            </text>
            <text
              x={2 * radius}
              y={radius + 20}
              text-anchor="middle"
              dominant-baseline="middle"
              fill="var(--color-subtext)"
              font-size="10"
              font-family="var(--font-code)"
            >
              {step.short}
            </text>
          </g>

          {#if i > 0}
            <line
              x1={radius * (4 * i - 1)}
              y1="75"
              x2={radius * (4 * i + 1)}
              y2="75"
              stroke={color}
              stroke-width="2"
            />
          {/if}
        {/each}
      </svg>

      <div
        class="w-full flex-1 my-8 border-1 border-emphasis-1 p-4 rounded-lg flex flex-col"
      >
        <div class="flex gap-2 items-center text-emphasis-1 mb-2">
          <div
            class="text-emphasis-2 bg-emphasis-2/25 py-1 text-[12px] text-center h-[24px] aspect-square rounded-full"
          >
            {selected_step + 1}
          </div>
          <span class="text-2xl font-mono">
            {steps[selected_step].title}
          </span>
        </div>
        <p class="text-subtext">
          {@html steps[selected_step].description ||
            "Description coming soon..."}
        </p>
        <div
          class="flex-1 w-full mt-4 border-1 border-emphasis-1 rounded-lg"
          id="step-container"
        >
          <div id="inner-step" class="w-full h-full">
            {#if selected_step === 0}
              <Font />
            {:else if selected_step === 1}
              <HTTP />
            {/if}
          </div>
        </div>
      </div>
    </div>
  </div>
</div>
