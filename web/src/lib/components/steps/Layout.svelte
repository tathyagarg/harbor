<script lang="ts">
  import { primaryColor, primaryTextColor } from "$lib";
  import { Mermaid } from "@friendofsvelte/mermaid";

  import mermaid from "mermaid";
  import { onMount } from "svelte";

  mermaid.initialize({
    startOnLoad: true,
    htmlLabels: true,
    theme: "base",
    themeVariables: {
      primaryColor,
      primaryTextColor,
      tertiaryColor: "#13293d",
    },
    flowchart: {
      useMaxWidth: true,
    },
  });

  onMount(() => {
    mermaid.contentLoaded();
  });
</script>

<div
  class="grid grid-cols-7 gap-8 items-center justify-center w-full h-full p-8 mx-auto"
>
  <div class="col-span-3 flex flex-col items-center">
    <h1 class="self-start">DOM Structure</h1>
    <Mermaid
      string={`
    graph TD
      A[Document] --> B[HTML]
      B --> C{Head}
      B --> D{Body}
      C --> E[Title]
      C --> F[Meta]
      D --> G[Headings]
      D --> H[Paragraphs]
      D --> I[etc.]
  `}
      config={{
        theme: "base",
        themeVariables: {
          primaryColor,
          primaryTextColor,
        },
      }}
    />
  </div>

  <span class="text-6xl text-(--text) text-center">→</span>

  <div class="col-span-3 flex flex-col items-center">
    <h1 class="self-start">Box Layout Structure</h1>
    <pre class="mermaid">
flowchart TD
  subgraph "Body"
    direction TD
  A["`Heading (h1)
pos(x1, y1)
dims(w1, h1)`"] ~~~ B["`Paragraph (p)
pos(x2, y2)
dims(w2, h2)`"]
  end

</pre>
  </div>
</div>
