<script setup lang="ts">
import { inject } from 'vue'
import type { Ref } from 'vue'

// ----------------------------------------------------------
// 下拉菜单触发器
// ----------------------------------------------------------

const isOpen = inject<Ref<boolean>>('dropdownOpen')!
const closeMenu = inject<() => void>('dropdownClose')!

function toggle() {
  if (isOpen.value) {
    closeMenu()
  } else {
    // force change via parent - we need a setter
    ;(isOpen as any).value = !isOpen.value
  }
}
</script>

<template>
  <div @click.stop="toggle" class="cursor-pointer">
    <slot />
  </div>
</template>
