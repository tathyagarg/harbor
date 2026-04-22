<script lang="ts">
  import { animate, onScroll, stagger } from "animejs";
  import { onMount } from "svelte";
  import Window from "../05_CSS/Window.svelte";
  import ExtractedUrl from "../05_CSS/ExtractedUrl.svelte";
  import Agent from "../05_CSS/Agent.svelte";

  let htmlLines: HTMLDivElement;
  let htmlCode = $state([
    { line: "<!DOCTYPE html>", emph: false, link: false },
    { line: "<html lang='en'>", emph: false, link: false },
    { line: "  <head>", emph: false, link: false },
    { line: "    <meta charset='UTF-8' />", emph: false, link: false },
    {
      line: "    <link rel='stylesheet' href='styles.css' />",
      emph: false,
      link: true,
    },
    {
      line: "    <link rel='stylesheet' href='/assets/styles.css' />",
      emph: false,
      link: true,
    },
    { line: "    <title>Document</title>", emph: false, link: false },
    { line: "  </head>", emph: false, link: false },
    { line: "  <body>", emph: false, link: false },
    { line: "    <h1>Hello World</h1>", emph: false, link: false },
    { line: "  </body>", emph: false, link: false },
    { line: "</html>", emph: false, link: false },
  ]);

  let extractedUrls: { url: string; state: string }[] = $state([]);

  onMount(async () => {
    await animate("#scanner", {
      opacity: 0,
      duration: 0,
    });

    await animate(".line", {
      opacity: 0,
      duration: 0,
    });

    await animate("#scanner", {
      autoplay: onScroll({
        target: "#pipeline",
        container: document.getElementsByName("html")[0],
      }),
    });

    await new Promise((resolve) => setTimeout(resolve, 1000));

    await animate(".line", {
      translateX: [-20, 0],
      opacity: [0, 1],
      duration: 200,
      delay: stagger(50, { start: 500 }),
      ease: "linear",
    })
      .then(async () => {
        await animate("#scanner", {
          opacity: 1,
          duration: 500,
          delay: 500,
        });
      })
      .then(async () => {
        await animate("#scanner", {
          opacity: 1,
          translateY: [0, htmlLines.clientHeight],
          duration: 1000,
        }).then(async () => {
          await animate("#scanner", {
            opacity: 0,
            duration: 500,
          });
        });
      })
      .then(() => {
        htmlCode = htmlCode.map((line) => {
          if (line.link) {
            return { ...line, emph: true };
          }
          return line;
        });

        extractedUrls = htmlCode
          .filter((line) => line.link)
          .map((line) => {
            const match = line.line.match(/href=['"]([^'"]+)['"]/);
            return match ? { url: match[1], state: "pending" } : null;
          })
          .filter((url) => url !== null) as { url: string; state: string }[];
      });

    await animate(".emph", {
      translateX: [0, 100, 0],
      duration: 250,
      delay: 250,
    });

    await animate(".exurl", {
      opacity: [0, 1],
      translateY: [20, 0],
      duration: 500,
      delay: stagger(100),
    }).then(() => {
      extractedUrls = extractedUrls.map((url) => ({
        ...url,
        state: "fetching",
      }));
    });

    await animate(".agent", {
      opacity: [0, 1],
      translateY: [20, 0],
      duration: 500,
      delay: stagger(100),
    }).then(() => {
      setTimeout(() => {
        extractedUrls = extractedUrls.map((url) => ({
          ...url,
          state: "done",
        }));
      }, 2250);
    });
  });
</script>

<div
  class="w-full h-full grid grid-cols-2 grid-rows-2 place-items-center p-4 gap-2"
>
  <div class="w-full h-full row-span-2 flex items-center">
    <Window title="index.html" hasDots>
      <div
        id="scanner"
        class="absolute top-0 left-0 right-0 h-[12px] bg-pretty-blue/20"
      ></div>

      <div class="h-full" bind:this={htmlLines}>
        {#each htmlCode as line}
          <div
            class={`relative text-[12px] font-mono text-emphasis-1/80 line 
            overflow-hidden text-ellipsis whitespace-nowrap
            ${line.emph ? "emph bg-pretty-blue/10" : ""} transition-all duration-500`}
            class:text-pretty-blue={line.link}
          >
            <span class="overflow-hidden text-ellipsis whitespace-nowrap">
              {line.line}
            </span>
          </div>
        {/each}
      </div>
    </Window>
  </div>

  <div class="w-full h-full col-start-2 row-start-1 flex items-center">
    <Window title="Extracted URLs">
      {#each extractedUrls as url}
        <ExtractedUrl url={url.url} state={url.state} />
      {/each}
    </Window>
  </div>

  <div class="w-full h-full col-start-2 row-start-2 flex items-center">
    <Window title="HTTP Agent">
      {#each extractedUrls as url}
        <Agent url={url.url} />
      {/each}
    </Window>
  </div>
</div>
