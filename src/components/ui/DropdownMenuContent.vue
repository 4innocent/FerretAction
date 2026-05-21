<script setup lang="ts">
import { inject, ref, onMounted, onUnmounted } from 'vue'
import type { Ref } from 'vue'

// ----------------------------------------------------------
// 下拉菜单内容面板
// ----------------------------------------------------------

defineProps<{
  align?: 'start' | 'end'
  side?: 'left' | 'right'
}>()

const isOpen = inject<Ref<boolean>>('dropdownOpen')!
const closeMenu = inject<() => void>('dropdownClose')!

const contentRef = ref<HTMLElement>()

function closeOnClickOutside(e: MouseEvent) {
  if (contentRef.value && !contentRef.value.contains(e.target as Node)) {
    closeMenu()
  }
}

onMounted(() => document.addEventListener('click', closeOnClickOutside, true))
onUnmounted(() => document.removeEventListener('click', closeOnClickOutside, true))
</script>

<template>
  <div
    v-if="isOpen"
    ref="contentRef"
    class="absolute z-50 mt-1 min-w-[180px] rounded-md border border-border bg-card
           p-1 shadow-md"
    :class="[
      align === 'end' ? 'right-0' : 'left-0',
      side === 'right' ? 'left-full top-0 ml-1' : ''
    ]"
    @click.stop="closeMenu()"
  >
    <slot />
  </div>
</template>
