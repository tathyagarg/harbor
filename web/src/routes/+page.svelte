<script lang="ts">
  import Components from "$lib/components/Components.svelte";
  import Stat from "$lib/components/Stat.svelte";
  import Icon from "@iconify/svelte";
  import { animate, cubicBezier, onScroll, stagger } from "animejs";
  import { onMount } from "svelte";

  onMount(() => {
    const elems = document.querySelectorAll(".animate");

    animate(elems, {
      opacity: [0, 1],
      translateY: [20, 0],
      filter: ["blur(10px)", "blur(0)"],
      duration: 300,
      delay: stagger(200),
      ease: cubicBezier(0.25, 0.1, 0.25, 1),
      autoplay: onScroll({
        target: "#header",
        debug: false,
      }),
    });
  });

  let component_datas = [
    {
      icon: "</>",
      title: "HTML Parser",
      description:
        "A custom HTML parser that converts HTML text into a DOM tree structure.",
      loc: "7,900",
    },
    {
      icon: "{ }",
      title: "CSS Parser",
      description:
        "A custom CSS parser that converts CSS text into a stylesheet object.",
      loc: "5,900",
    },
    {
      icon: "Aa",
      title: "TrueType Font Reader",
      description:
        "A custom TrueType font reader that reads .ttf files into usable glyph data for rendering text.",
      loc: "4,300",
    },
    {
      icon: "↔",
      title: "HTTP Client",
      description:
        "A custom HTTP client that handles all network requests, including fetching HTML, CSS, etc. It supports upto HTTP/1.1 in writing and HTTP/2 in reading (i think).",
      loc: "2,200",
    },
    {
      icon: "░",
      title: "Renderer",
      description:
        "A custom renderer that takes the DOM tree and stylesheet object and renders it to the screen.",
      loc: "2,200",
    },
    {
      icon: "{ }",
      title: "JavaScript Engine",
      description:
        "A custom JavaScript engine written in Zig that executes JavaScript code on web pages. (Not functional yet)",
      loc: "2,400",
    },
  ];
</script>

<div
  class="min-h-screen flex flex-col items-center justify-center pt-32 pb-16"
  id="header"
>
  <div class="font-serif text-9xl text-center text-(white/80) my-4">
    <h1 class="animate">welcome to</h1>
    <h1 class="italic text-(--celadon) animate">harbor browser</h1>
  </div>
  <h2 class="text-center animate">
    A web browser written completely from scratch.<br />Every service
    handwritten in Rust (and Zig)
  </h2>
  <div class="flex gap-4 my-4">
    <a
      class="bg-(--baltic-blue) p-4 rounded-xl animate"
      href="https://github.com/tathyagarg/harbor/releases"
    >
      Alpha is now available!
    </a>
    <a
      href="https://github.com/tathyagarg/harbor"
      class="text-(--dark-deep-space-blue) bg-(--celadon) p-4 rounded-xl animate"
    >
      <Icon icon="mdi:github" class="inline-block" />
      GitHub
    </a>
  </div>
  <div class="w-[80%] rounded-lg">
    <img
      src="assets/ss.png"
      alt="screenshot of harbor browser"
      class="animate shadow-lg mt-8 overflow-hidden rounded-xl"
    />
  </div>
</div>

<hr class="border-(--celadon)" />

<div class="flex justify-around py-16">
  <Stat title="Lines of Code" value="33k+" />
  <Stat title="Third Party Dependencies" value="0" />
  <Stat title="Core Modules" value="6" />
</div>

<hr class="border-(--celadon)" />

<div class="py-16">
  <h1 class="text-6xl text-center py-4">Core Components</h1>
  <p class="text-center">
    The core components that make up Harbor Browser.<br /> Each written by hand in
    Rust (except JavaScript, that's in Zig)
  </p>

  <div class="grid grid-cols-3 gap-8 mt-8">
    {#each component_datas as data}
      <Components
        icon={data.icon}
        title={data.title}
        description={data.description}
        loc={data.loc}
      />
    {/each}
  </div>
</div>
