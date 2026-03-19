<script lang="ts">
  import "./layout.css";
  import Icon from "@iconify/svelte";

  import { onMount } from "svelte";
  import Navbar from "$lib/components/Navbar.svelte";

  onMount(() => {
    const canvas = document.createElement("canvas");
    const size = 400;
    canvas.width = size;
    canvas.height = size;
    const ctx = canvas.getContext("2d");

    const imageData = ctx?.createImageData(size, size);
    const data = imageData?.data;

    for (let i = 0; i < (data?.length ?? 0); i += 4) {
      const shade_range = Math.random();
      const shade = (shade_range > 0.5 ? 255 : 0) * Math.random() * 0.25;

      if (data) {
        data[i] = shade;
        data[i + 1] = shade;
        data[i + 2] = shade;
        data[i + 3] = 20;
      }
    }

    ctx?.putImageData(imageData ?? new ImageData(0, 0), 0, 0);
    document.body.style.backgroundImage = `url(${canvas.toDataURL()})`;
  });

  const { children } = $props();
</script>

<svelte:head>
  <title>Harbor Browser</title>

  <link rel="icon" href="/assets/favicon.png" />
</svelte:head>

<div class="w-[80%] mx-auto">
  <Navbar />
  {@render children()}
</div>

<div class="h-[50vh] bg-(--accent-text) relative overflow-hidden">
  <div
    class="w-[80%] mx-auto flex flex-col gap-4 h-full text-baltic-blue py-12"
  >
    <div class="pb-4">
      <h1 class="text-6xl font-bold">Harbor Browser</h1>
      <p>A browser built completely from scratch in Rust and Zig.</p>
    </div>

    <div>
      <h2 class="font-bold text-lg">Resources</h2>
      <a href="https://github.com/tathyagarg/harbor" class="hover:underline">
        <Icon icon="mdi:github" class="inline-block" />
        GitHub</a
      >
    </div>

    <!-- made with love tag -->
    <h2 class="text-sm text-center text-(--baltic-blue) mt-auto">
      Made with ❤️ by <a href="https://arson.dev/" class="underline">Tathya</a>
    </h2>
  </div>

  <img
    src="/assets/favicon.png"
    alt="Harbor Browser Logo"
    class="w-128 absolute -bottom-32 -right-32"
  />
</div>
