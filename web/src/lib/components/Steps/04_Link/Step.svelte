<script lang="ts">
  import { animate, createTimeline, onScroll, stagger } from "animejs";
  import { onMount } from "svelte";

  let link_data = $state([
    {
      name: "<link rel='stylesheet' href='https://example.com/assets/example.css'>",
      detail: "",
    },
    { name: "<link rel='stylesheet' href='/assets/style.css'>", detail: "" },
  ]);

  let y_offsets = $derived(
    link_data.reduce((acc, curr, index) => {
      console.log(index, acc, link_data[index - 1]);

      acc.push(
        (index > 0 ? acc[index - 1] + 17.5 : 15) +
          (index > 0 && link_data[index - 1].detail.length > 0 ? 10 : 0),
      );
      return acc;
    }, [] as number[]),
  );

  let total = $derived(
    y_offsets[y_offsets.length - 1] +
      (link_data[link_data.length - 1].detail.length > 0 ? 10.0 : 0) +
      12.5,
  );

  console.log(y_offsets);

  onMount(() => {
    let timeline = createTimeline({
      autoplay: onScroll({
        target: "#pipeline",
        container: document.getElementsByName("html")[0],
      }),
      defaults: {
        duration: 500,
        ease: "easeInOutQuad",
      },
    });

    animate([".link-line-box", ".link-line"], {
      translateX: -10,
      opacity: 0,
      duration: 0,
    });

    timeline
      .add(
        ".link-line-box",
        {
          translateX: 0,
          opacity: 1,
          delay: stagger(100),
        },
        500,
      )
      .add(
        ".link-line",
        {
          translateX: 0,
          opacity: 1,
          delay: stagger(100),
        },
        500,
      )
      .add(
        ".link-line",
        {
          translateX: 10,
          opacity: 0,
          delay: stagger(100),
        },
        2000,
      )
      .call(() => {
        link_data = [
          { name: "https://example.com/assets/example.css", detail: "hi" },
          { name: "/assets/style.css", detail: "" },
        ];
      })
      .add(
        ".link-line",
        {
          translateX: [-10, 0],
          opacity: [0, 1],
          delay: stagger(100),
        },
        2750,
      );
  });
</script>

<div class="w-full h-full flex items-center justify-center px-24 text-subtext">
  <svg viewBox="0 0 400 200" class="font-code">
    <rect
      x="5"
      y="5"
      width="390"
      height="190"
      fill="transparent"
      stroke="rgba(from var(--color-emphasis-1) r g b / 25%)"
      stroke-width="1"
      rx="8"
      ry="8"
    />

    <rect
      x="10"
      y="10"
      width="380"
      height={total}
      fill="rgba(from var(--color-text-dark) r g b / 25%)"
      stroke="rgba(from var(--color-emphasis-1) r g b / 25%)"
      stroke-width="1"
      rx="2"
      ry="2"
    />

    {#each link_data as link, index}
      <rect
        class={`link-line-box link-line-box-${index}`}
        x="15"
        y={y_offsets[index]}
        width="370"
        height={17.5 + (link.detail.length > 0 ? 10 : 0)}
        fill="transparent"
        stroke="rgba(from var(--color-emphasis-1) r g b / 25%)"
        stroke-width="0.5"
      />

      <g class={`link-line link-line-${index}`}>
        <text
          x="20"
          y={y_offsets[index] + 12.5}
          fill="var(--color-subtext)"
          font-size="8">{link.name}</text
        >

        {#if link.detail.length > 0}
          <text
            class="link-line"
            x="20"
            y={y_offsets[index] + 22.5}
            fill="var(--color-emphasis-3)"
            font-size="6">{link.detail}</text
          >
        {/if}
      </g>
    {/each}
  </svg>
</div>
