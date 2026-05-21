<script setup lang="ts">
// ----------------------------------------------------------
// 下拉菜单项
// ----------------------------------------------------------
</script>

<template>
  <button
    class="relative flex w-full cursor-pointer select-none items-center rounded-sm px-2 py-1.5
           text-sm outline-none transition-colors hover:bg-secondary hover:text-foreground
           data-[destructive]:text-destructive"
  >
    <slot />
  </button>
</template>
