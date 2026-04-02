<script lang="ts">
  import "./layout.css";
  import favicon from "$lib/assets/favicon.svg";
  import { onMount } from "svelte";

  let { children } = $props();

  onMount(() => {
    const canvas = document.createElement("canvas");
    const size = 400;
    canvas.width = size;
    canvas.height = size;
    const ctx = canvas.getContext("2d");

    const imageData = ctx?.createImageData(size, size)!;
    const data = imageData?.data ?? [];

    for (let i = 0; i < data.length; i += 4) {
      const shade_range = Math.random();
      const shade = (shade_range > 0.5 ? 255 : 0) * Math.random() * 0.75;

      data[i] = shade;
      data[i + 1] = shade;

      data[i + 3] = 10;
    }

    ctx?.putImageData(imageData, 0, 0);
    document.body.style.backgroundImage = `url(${canvas.toDataURL()})`;
  });
</script>

<svelte:head>
  <title>Harbor Browser</title>
  <link rel="icon" href={favicon} />
</svelte:head>
{@render children()}
