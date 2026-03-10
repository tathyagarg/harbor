<script>
  import { Mermaid } from "@friendofsvelte/mermaid";
</script>

<div class="relative w-full top-[7.5%]">
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
        primaryColor: "#3b82f6",
        primaryTextColor: "#E6F1F7",
      },
      sequence: {},
    }}
  />
</div>
