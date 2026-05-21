<script setup lang="ts">
import { inject } from 'vue'
import type { Ref } from 'vue'

// ----------------------------------------------------------
// Tabs 内容面板
// ----------------------------------------------------------

defineProps<{
  value: string
}>()

const activeTab = inject<Ref<string>>('tabsActiveValue')!
</script>

<template>
  <div v-if="activeTab === value" class="flex-1 min-h-0">
    <slot />
  </div>
</template>
