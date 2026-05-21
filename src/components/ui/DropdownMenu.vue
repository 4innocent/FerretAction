<script setup lang="ts">
import { ref, provide, readonly } from 'vue'
import type { Ref } from 'vue'

// ----------------------------------------------------------
// 下拉菜单组件
// 用法:
//   <DropdownMenu>
//     <DropdownMenuTrigger><button>...</button></DropdownMenuTrigger>
//     <DropdownMenuContent>
//       <DropdownMenuItem @click="...">选项</DropdownMenuItem>
//     </DropdownMenuContent>
//   </DropdownMenu>
// ----------------------------------------------------------

const isOpen = ref(false)
provide('dropdownOpen', readonly(isOpen) as Ref<boolean>)
provide('dropdownClose', () => { isOpen.value = false })

// 键盘关闭
function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') isOpen.value = false
}

import { onMounted, onUnmounted } from 'vue'

onMounted(() => document.addEventListener('keydown', onKeydown))
onUnmounted(() => document.removeEventListener('keydown', onKeydown))
</script>

<template>
  <div class="relative inline-flex">
    <slot />
  </div>
</template>
