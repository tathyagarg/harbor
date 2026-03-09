<script lang="ts">
  import { animate, onScroll, stagger } from "animejs";
  import { onMount } from "svelte";

  onMount(() => {
    const elems = document.querySelectorAll(".animate");

    animate(elems, {
      opacity: [0, 1],
      translateY: [20, 0],
      filter: ["blur(10px)", "blur(0)"],
      duration: 300,
      delay: stagger(200),
      ease: "cubicBezier(0.25, 0.1, 0.25, 1)",
      autoplay: onScroll({
        target: "#header",
        debug: false,
      }),
    });
  });
</script>

<div
  class="min-h-screen flex flex-col items-center justify-center w-[80%] m-auto py-32"
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
    <button class="bg-(--baltic-blue) p-4 rounded-xl animate"
      >Alpha is now available!</button
    >
    <button
      class="text-(--dark-deep-space-blue) bg-(--celadon) p-4 rounded-xl animate"
      >GitHub</button
    >
  </div>
  <img
    src="/assets/ss.png"
    alt="screenshot of harbor browser"
    class="animate rounded-lg shadow-lg"
  />
</div>

<div class="h-[200vh]"></div>
