<script setup lang="ts">
import { inject } from 'vue'

// ----------------------------------------------------------
// 可折叠面板触发器
// ----------------------------------------------------------

const toggle = inject<() => void>('collapsibleToggle')!
</script>

<template>
  <button type="button" @click="toggle">
    <slot />
  </button>
</template>
