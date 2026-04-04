<script lang="ts">
  import { animate, createTimeline, onScroll } from "animejs";
  import { onMount } from "svelte";

  let packetColor = $state("rgba(from var(--color-subtext) r g b / 25%)");
  let animationState = $state("client");

  const client_lines = [
    "GET /index.html HTTP/1.1",
    "Host: example.com",
    "User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)",
    "Accept: text/html",
  ];

  const server_lines = [
    "\nHTTP/1.1 200 OK\nContent-Type: text/html; charset=UTF-8\nContent-Length: 1256",
    "",
    "<!DOCTYPE html>\n<html>\n<head>\n<title>Example Domain</title>\n</head>\n<body>\n<h1>Example Domain</h1>",
    "<p>This domain is for use in illustrative examples in documents. You may use this\n    domain in literature without prior coordination or asking for permission.</p>\n</body>\n</html>",
  ];

  let client_lines_index = $state(0);
  let server_lines_index = $state(0);

  onMount(async () => {
    animate("#packet", {
      cx: 170,
      duration: 0,
    });

    let timeline = createTimeline({
      autoplay: onScroll({
        target: "#pipeline",
        container: document.getElementsByName("html")[0],
      }),
    });

    packetColor = "rgba(from var(--color-pretty-blue) r g b / 75%)";

    timeline
      .add("#packet", {
        cx: 280,
        duration: 1000,
        easing: "outCirc",
        loop: 4,
        delay: 1000,
        onLoop: (anim) => {
          let codeLine = document.createElement("div");
          codeLine.textContent = client_lines[client_lines_index++];

          animate(codeLine, {
            opacity: 0,
            translateX: -20,
            duration: 0,
          });

          code.appendChild(codeLine);

          animate(codeLine, {
            opacity: 1,
            translateX: 0,
            duration: 250,
            easing: "easeInOutQuad",
          });
        },
      })
      .call(() => {
        packetColor = "rgba(from var(--color-gh-green) r g b / 75%)";
        animationState = "server";
      })
      .add("#packet", {
        cx: 170,
        duration: 1000,
        easing: "outCirc",
        loop: 4,
        delay: 500,
        onLoop: (anim) => {
          let codeLine = document.createElement("div");
          codeLine.textContent = server_lines[server_lines_index++];

          animate(codeLine, {
            opacity: 0,
            translateX: 20,
            duration: 0,
          });

          code.appendChild(codeLine);

          animate(codeLine, {
            opacity: 1,
            translateX: 0,
            duration: 250,
            easing: "easeInOutQuad",
          });
        },
      });
  });

  let code: HTMLElement;
</script>

<div
  class="h-full w-full flex flex-col items-center justify-center px-24 text-subtext"
>
  <svg viewBox="0 0 450 200" class="w-2/3">
    <defs>
      <filter id="client-glow" x="-50%" y="-50%" width="200%" height="200%">
        <feDropShadow
          dx="0"
          dy="0"
          stdDeviation="2"
          flood-color="rgba(from var(--color-pretty-blue) r g b / 75%)"
          flood-opacity="1"
        />
      </filter>

      <filter id="server-glow" x="-50%" y="-50%" width="200%" height="200%">
        <feDropShadow
          dx="0"
          dy="0"
          stdDeviation="2"
          flood-color="rgba(from var(--color-gh-green) r g b / 75%)"
          flood-opacity="1"
        />
      </filter>
    </defs>

    <circle
      cx="170"
      cy="100"
      r="5"
      fill={packetColor}
      filter={`url(#${animationState}-glow)`}
      id="packet"
    />

    <g>
      <rect
        x="20"
        y="60"
        width="140"
        height="80"
        fill="rgba(from var(--color-pretty-blue) r g b / 15%)"
        stroke="var(--color-pretty-blue)"
        stroke-width="2"
        filter="url(#client-glow)"
        rx="8"
        ry="8"
      />
      <text
        x="90"
        y="100"
        font-size="18"
        class="text-center"
        dominant-baseline="middle"
        text-anchor="middle"
        fill="var(--color-pretty-blue)"
        font-family="var(--font-code)"
      >
        [ client ]
      </text>
    </g>

    <g>
      <rect
        x="290"
        y="60"
        width="140"
        height="80"
        fill="rgba(from var(--color-gh-green) r g b / 15%)"
        stroke="var(--color-gh-green)"
        stroke-width="2"
        filter="url(#server-glow)"
        rx="8"
        ry="8"
      />

      <text
        x="360"
        y="100"
        font-size="18"
        class="text-center"
        dominant-baseline="middle"
        text-anchor="middle"
        fill="var(--color-gh-green)"
        font-family="var(--font-code)"
      >
        [ server ]
      </text>
    </g>
  </svg>

  <pre class="w-full border-1 border-emphasis-1/25 rounded-lg text-xs p-2"><div
      bind:this={code}
      class="w-full max-h-20 overflow-y-scroll"></div></pre>
</div>
