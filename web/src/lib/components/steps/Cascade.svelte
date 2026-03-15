<script>
  import { primaryColor, primaryTextColor } from "$lib";
  import mermaid from "mermaid";
  import { onMount } from "svelte";

  mermaid.initialize({
    startOnLoad: true,
    htmlLabels: true,
    theme: "base",
    themeVariables: {
      primaryColor,
      primaryTextColor,
      secondaryColor: "#13293d",
      nodeTextColor: "#fff",
      tertiaryColor: "#8fb99680",
      titleColor: primaryTextColor,
    },
  });

  onMount(() => {
    mermaid.contentLoaded();
  });
</script>

<div class="relative w-full top-[7.5%] rounded-lg">
  <div class="w-[60%] mx-auto">
    <pre class="mermaid">
flowchart TD
  subgraph "body"
    H[font-size: 16px]
  end

  subgraph "div"
    A[position]
    B[color]
    C[font-size]
    G[width: 5em]
  end

  subgraph "span"
    D[position]
    E[color]
    F[font-size]
  end

  B -.->|Cascade| E
  C -.->|Cascade| F
  H -.->|Cascade| C
  H ---|Used| G
  G ==>|Resolve| I[width: 80px]

  classDef div color:black
  classDef span color:black
  classDef misc color:black

  class A,B,C,G div
  class D,E,F span
  class H,I misc
</pre>
  </div>
</div>
