<script lang="ts">
  import Icon from "@iconify/svelte";

  let {
    href,
    icon,
    color,
    text,
  }: {
    href: string;
    icon: string;
    color: string;
    text: string;
  } = $props();
</script>

<a
  class="mr-4 flex items-center justify-center border-1 rounded-full py-1 px-4 anim-3"
  style={`color: ${color}; background-color: rgb(from ${color} r g b / 15%); border-color: ${color};`}
  {href}
  target="_blank"
>
  <Icon {icon} class="text-2xl inline" />
  <span class="ml-2 text-sm">{text}</span>
</a>
