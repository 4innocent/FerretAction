<script setup lang="ts">
// ----------------------------------------------------------
// 分割线组件
// ----------------------------------------------------------

withDefaults(defineProps<{
  orientation?: 'horizontal' | 'vertical'
}>(), {
  orientation: 'horizontal',
})
</script>

<template>
  <div
    role="separator"
    class="shrink-0 bg-border"
    :class="orientation === 'horizontal' ? 'h-px w-full' : 'h-full w-px'"
  />
</template>
