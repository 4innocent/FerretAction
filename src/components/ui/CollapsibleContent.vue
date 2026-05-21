<script setup lang="ts">
import { inject } from 'vue'
import type { Ref } from 'vue'

// ----------------------------------------------------------
// 可折叠面板内容
// ----------------------------------------------------------

const isOpen = inject<Ref<boolean>>('collapsibleOpen')!
</script>

<template>
  <div v-if="isOpen">
    <slot />
  </div>
</template>
