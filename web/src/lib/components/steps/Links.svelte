<script>
  import { primaryColor, primaryTextColor } from "$lib";
  import { Mermaid } from "@friendofsvelte/mermaid";
</script>

<div class="relative w-full top-[7.5%] bg-(--cerulean)/75 rounded-lg">
  <Mermaid
    string={`
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
`}
    config={{
      theme: "base",
      themeVariables: {
        primaryColor,
        primaryTextColor,
      },
      sequence: {},
    }}
  />
</div>
