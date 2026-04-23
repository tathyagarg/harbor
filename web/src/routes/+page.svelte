<!-- svelte-ignore state_referenced_locally -->

<script lang="ts">
  import Stat from "$lib/components/Stat.svelte";
  import { animate, createTimeline, onScroll, stagger } from "animejs";
  import { onMount } from "svelte";
  import type { PageProps } from "./$types";

  import Font from "$lib/components/Steps/01_Font/Step.svelte";
  import HTTP from "$lib/components/Steps/02_HTTP/Step.svelte";
  import HTML from "$lib/components/Steps/03_HTML/Step.svelte";
  import Link from "$lib/components/Steps/04_Link/Step.svelte";
  import CSS from "$lib/components/Steps/05_CSS/Step.svelte";
  import Cascade from "$lib/components/Steps/06_Cascade/Step.svelte";

  import Test from "$lib/components/Steps/99_Test/Step.svelte";

  import Icon from "@iconify/svelte";

  let { data }: PageProps = $props();

  let stats = [
    {
      value: Math.floor(data.lines.total / 1000),
      title: "Lines of Code",
      suffix: "K+",
    },
    {
      value: Math.floor(data.file_count / 10) * 10,
      title: "Files (kinda)",
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

  let scrollPos = $state(0);

  onMount(() => {
    document.addEventListener("scroll", () => {
      scrollPos = Math.floor((window.scrollY / window.innerHeight) * 1.05);
    });

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
      .add(".anim-2", {
        opacity: [0, 1],
        translateY: [50, 0],
        delay: stagger(100, { start: 500 }),
      })
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
      delay: 500,
      duration: 500,
      ease: "inOutQuad",
      autoplay: onScroll({
        container: document.getElementsByName("html")[0],
      }),
    });

    animate("#architecture", {
      opacity: 0,
      translateY: 50,
      duration: 0,
    });

    animate("#architecture", {
      opacity: [0, 1],
      translateY: [50, 0],
      delay: 500,
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
      longDesc:
        "The Font Reader is responsible for reading and parsing font files. It processes the binary data of the font file and extracts various tables such as <code>cmap</code>, <code>glyf</code>, and others. Harbor currently supports 13 different tables.",
    },
    {
      short: "HTTP",
      title: "HTTP Client",
      description:
        "Fetches resources over the network using the HTTP protocol.",
      longDesc:
        "The HTTP Client is responsible for fetching resources from the network using the HTTP protocol. It handles DNS resolution, establishes connections, sends requests, and parses responses. Harbor's HTTP client uses <code>rustls</code> for TLS support.",
    },
    {
      short: "HTML",
      title: "HTML Parser",
      description: "Parses HTML documents and constructs the DOM tree.",
      longDesc:
        "The HTML Parser takes raw HTML text and parses it according to the HTML5 specification. It constructs a Document Object Model (DOM) tree that represents the structure of the HTML document. The parser handles various edge cases and quirks of HTML parsing to ensure compatibility with real-world web content.",
    },
    {
      short: "Links",
      title: "Link Resolver",
      description: "Resolves links between resources.",
      longDesc:
        "The Link Resolver is responsible for resolving links between resources. It takes care of resolving relative URLs, handling redirects, and managing the relationships between different resources on a webpage.",
    },
    {
      short: "CSS",
      title: "CSS Parser",
      description: "Parses CSS stylesheets and constructs the CSSOM tree.",
      longDesc:
        "The CSS Parser processes CSS stylesheets and constructs a CSS Object Model (CSSOM) tree. It handles the syntax of CSS, including selectors, properties, and values.",
    },
    { short: "Cascade", title: "Cascade & Inheritance" },
    { short: "Layout", title: "Layout Engine" },
    { short: "Rasterize", title: "Rasterizer" },
    { short: "Paint", title: "Paint Engine" },
  ];

  const radius = 25;

  let selected_step = $state(0);

  async function switchTo(n: number) {
    if (n === selected_step) return;

    animate("#inner-step", {
      opacity: [1, 0],
      translateX: [0, -200],
      duration: 250,
      easing: "easeInOutQuad",
    });

    await animate("#step-content", {
      opacity: [1, 0],
      duration: 250,
      easing: "easeInOutQuad",
    });

    animate("#inner-step", {
      translateX: 200,
      duration: 0,
    });

    selected_step = n;

    animate("#step-content", {
      opacity: [0, 1],
      duration: 250,
      easing: "easeInOutQuad",
    });

    animate("#inner-step", {
      opacity: [0, 1],
      translateX: [200, 0],
      duration: 250,
      easing: "easeInOutQuad",
    });
  }

  let labelEls: HTMLElement[] = [];
  let items = [
    { label: "The Pipeline", href: "#pipeline" },
    { label: "Architecture", href: "#architecture" },
  ];

  function expand() {
    labelEls.forEach((el) => {
      const w = el.scrollWidth;
      el.style.width = w + "px";
      el.style.opacity = "1";
    });
  }

  function collapse() {
    labelEls.forEach((el) => {
      el.style.width = "0px";
      el.style.opacity = "0";
    });
  }
</script>

<div
  class="overflow-hidden transition-all duration-300 fixed top-1/2 left-4 -translate-y-1/2
  flex flex-col items-start gap-1 z-10 border-1 border-emphasis-1/25 rounded-lg px-2 py-4 font-body
  cursor-pointer text-sm backdrop-blur-sm shadow-2xl shadow-black"
  onmouseenter={expand}
  onmouseleave={collapse}
  role="navigation"
>
  {#each items as item, i}
    {@const selected = scrollPos == i + 1}
    <button
      class={`flex items-center whitespace-nowrap gap-2 tracking-wider
      rounded-md p-2 duration-300 transition-all w-full
      ${selected ? "bg-emphasis-2/25" : "hover:bg-emphasis-1/25"}`}
    >
      <span
        class="shrink-0"
        class:text-emphasis-2={selected}
        class:text-subtext={!selected}>0{i + 1}</span
      >

      <a
        bind:this={labelEls[i]}
        class="overflow-hidden transition-all duration-300"
        class:text-subtext={!selected}
        style="width: 0px; opacity: 0"
        href={item.href}
      >
        {item.label}
      </a>
    </button>
  {/each}
</div>

<div class="w-[90%] mx-auto h-screen z-2 p-12">
  <h1 class="text-[15rem]/[15rem] text-emphasis-2">HARBOR BROWSER</h1>

  <p class="text-subtext anim w-[70%]">
    Harbor Browser is a custom web browser engine built from scratch with Rust
    and Zig. <span class="text-text"
      >Every core system - from networking to parsing to layouts to rendering,
      and more - was implemented manually without relying on existing browser
      engines or frameworks</span
    >
  </p>

  <div class="grid grid-cols-4 gap-12 mt-12">
    {#each stats as stat}
      <div class="anim-2">
        <Stat {...stat} />
      </div>
    {/each}
  </div>
</div>

<div class="w-[50%] mx-auto" id="page">
  <div class="h-screen" id="pipeline">
    <div class="h-full py-[5%] flex flex-col">
      <div class="flex gap-4 items-baseline my-2">
        <span class="text-emphasis-1">01</span>
        <div class="w-full bg-emphasis-1 h-0.5"></div>
      </div>
      <h1 class="text-emphasis-2 text-6xl my-4">The Pipeline</h1>
      <p class="text-subtext">
        Harbor Browser's architecture is divided into several core systems that
        work together to process and render web content.
      </p>

      <svg class="mx-auto w-full" viewBox="0 0 900 200">
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

        <path
          d={`M ${6 * radius} ${4 * radius} 
            C ${6 * radius} ${5 * radius + 75}, 
              ${14 * radius} ${5 * radius + 75}, 
              ${14 * radius} ${4 * radius}
            `}
          stroke="rgba(from var(--color-emphasis-1) r g b / 25%)"
          stroke-dasharray="4 4"
          stroke-width="2"
          fill="none"
        />

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
              {i + 1}
            </text>
            <text
              x={2 * radius}
              y={radius + 20}
              text-anchor="middle"
              dominant-baseline="middle"
              fill={color}
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
        class="h-full w-full flex-1 mt-8 border-1 border-emphasis-1/25 rounded-lg flex flex-col"
      >
        <div class="h-full w-full p-4 flex flex-col">
          <div
            class="flex gap-2 items-center text-emphasis-1 mb-2"
            id="step-content"
          >
            <div
              class="text-emphasis-2 bg-emphasis-2/25 py-1 text-[12px] text-center h-[24px] aspect-square rounded-full"
            >
              {selected_step + 1}
            </div>
            <span class="text-2xl font-mono">
              {steps[selected_step].title}
            </span>
          </div>
          <p class="text-subtext" id="step-content">
            {@html steps[selected_step].description ||
              "Description coming soon..."}
          </p>
          <div
            class="flex-1 w-full mt-4 border-1 border-emphasis-1/25 rounded-lg"
            id="step-container"
          >
            <div id="inner-step" class="w-full h-full">
              {#if selected_step === 0}
                <Font />
              {:else if selected_step === 1}
                <HTTP />
              {:else if selected_step === 2}
                <HTML />
              {:else if selected_step === 3}
                <Link />
              {:else if selected_step === 4}
                <CSS />
              {:else if selected_step === 5}
                <Cascade />
              {:else if selected_step === 6}
                <Test />
              {/if}
            </div>
          </div>
        </div>

        <div id="step-content">
          {#if steps[selected_step].longDesc}
            <div
              class="mt-4 px-4 py-2 border-t-1 border-emphasis-1/25 rounded-b-lg text-sm max-h-16 overflow-y-scroll"
            >
              <p class="text-subtext">
                {@html steps[selected_step].longDesc}
              </p>
            </div>
          {/if}
        </div>

        <div class="flex items-center justify-center gap-4 my-4">
          <button
            class="cursor-pointer p-2 rounded-md bg-emphasis-1/25 disabled:cursor-not-allowed disabled:opacity-50 hover:bg-emphasis-1/50 transition-colors duration-300"
            onclick={() => switchTo(selected_step - 1)}
            disabled={selected_step === 0}
          >
            <Icon icon="mdi:arrow-left" width="20" height="20" />
          </button>
          <span>{selected_step + 1}/{steps.length}</span>
          <button
            class="cursor-pointer p-2 rounded-md bg-emphasis-1/25 disabled:cursor-not-allowed disabled:opacity-50 hover:bg-emphasis-1/50 transition-colors duration-300"
            onclick={() => switchTo(selected_step + 1)}
            disabled={selected_step === steps.length - 1}
          >
            <Icon icon="mdi:arrow-right" width="20" height="20" />
          </button>
        </div>
      </div>
    </div>
  </div>

  <div class="h-screen" id="architecture">
    <div class="h-full py-[5%] flex flex-col">
      <div class="flex gap-4 items-baseline my-2">
        <span class="text-emphasis-1">02</span>
        <div class="w-full bg-emphasis-1 h-0.5"></div>
      </div>
      <h1 class="text-emphasis-2 text-6xl my-4">Architecture</h1>
      <p class="text-subtext">Explore what's inside Harbor Browser.</p>
    </div>
  </div>
</div>

<div class="h-[40vh] w-full bg-emphasis-1 text-text-dark relative">
  <div
    class="absolute top-0 left-0 w-full h-full overflow-hidden pointer-events-none"
  >
    <img
      src="/circuit.png"
      alt="Circuit"
      class="object-contain opacity-10 -rotate-45 absolute left-0"
    />

    <img
      src="/circuit.png"
      alt="Circuit"
      class="object-contain opacity-10 rotate-45 absolute right-0 top-1/2"
    />
  </div>

  <div class="w-1/2 mx-auto h-full flex justify-between py-16">
    <div class="flex flex-col gap-4">
      <h2 class="text-2xl">Project</h2>
      <a
        href="https://github.com/tathyagarg/harbor"
        class="underline hover:decoration-wavy">GitHub</a
      >
      <a
        href="https://github.com/tathyagarg/harbor/issues"
        class="underline hover:decoration-wavy">Issues</a
      >
    </div>
    <div>
      Under the <a
        href="https://github.com/tathyagarg/harbor/blob/main/LICENSE"
        class="underline hover:decoration-wavy">MIT License</a
      >.
    </div>

    <div class="flex flex-col gap-4">
      <h2 class="text-2xl">Making</h2>
      <p>
        Made with <span class="text-red-500">❤</span> by
        <a href="https://arson.dev" class="underline hover:decoration-wavy"
          >Tathya</a
        >.
      </p>

      <p>
        Built for <a
          href="https://flavortown.hackclub.com/"
          class="underline hover:decoration-wavy">Hack Club's Flavortown</a
        >.
      </p>
    </div>
  </div>
</div>
