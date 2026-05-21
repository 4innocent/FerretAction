<script setup lang="ts">
import { ref } from 'vue'

// ----------------------------------------------------------
// 工具提示组件
// 用法: <Tooltip text="提示文字"><button>...</button></Tooltip>
// ----------------------------------------------------------

defineProps<{
  text: string
}>()

const isVisible = ref(false)
let hideTimer: ReturnType<typeof setTimeout>

function show() {
  clearTimeout(hideTimer)
  isVisible.value = true
}

function hide() {
  hideTimer = setTimeout(() => {
    isVisible.value = false
  }, 100)
}
</script>

<template>
  <div class="relative inline-flex" @mouseenter="show" @mouseleave="hide">
    <slot />
    <div
      v-if="isVisible"
      class="absolute left-1/2 bottom-full mb-1.5 -translate-x-1/2 z-50
             rounded-md border border-border bg-card px-2.5 py-1 text-xs text-foreground
             shadow-md pointer-events-none whitespace-nowrap"
    >
      {{ text }}
    </div>
  </div>
</template>
