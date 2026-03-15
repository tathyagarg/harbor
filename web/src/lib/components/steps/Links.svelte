<script>
  import { primaryTextColor } from "$lib";
  import mermaid from "mermaid";
  import { onMount } from "svelte";

  mermaid.initialize({
    startOnLoad: true,
    htmlLabels: true,
    theme: "base",
    themeVariables: {
      primaryColor: "#13293d",
      primaryTextColor: "#fff",
      secondaryColor: "#13293d",
      nodeTextColor: "#fff",
      tertiaryColor: "#8fb99680",
      titleColor: primaryTextColor,
    },
    sequence: {
      useMaxWidth: true,
    },
  });

  onMount(() => {
    mermaid.contentLoaded();
  });
</script>

<div class="relative w-full rounded-lg h-full flex items-center justify-center">
  <div class="w-[60%] mx-auto">
    <pre class="mermaid">
sequenceDiagram
  participant Client
  participant Engine
  participant ProcessQueue
  participant Server

  Client->>Engine: Request Resource
  Engine->>Server: Fetch Resource
  Server-->>Engine: Return Resource
  Engine->>ProcessQueue: Enqueue Resource
  ProcessQueue-->>Engine: Dequeue Resource
  Engine->>Client: Use Resource
  </pre>
  </div>
</div>
