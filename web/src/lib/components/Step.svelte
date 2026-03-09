<script lang="ts">
  export let title: string;
  export let onClick: () => void;

  export let last: boolean = false;
</script>

<li>
  <button
    on:click={onClick}
    class="bg-(--cerulean) py-2 pl-8 border-2 border-(--celadon)/25
    rounded-lg relative left-4 cursor-pointer hover:bg-(--cerulean)/75
    transition-colors duration-200 w-full text-left"
    type="button"
    tabindex="0"
    class:mb-4={!last}
  >
    {title}
  </button>
</li>
