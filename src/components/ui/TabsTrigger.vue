<script setup lang="ts">
import { inject, computed } from 'vue'
import type { Ref } from 'vue'

// ----------------------------------------------------------
// Tabs 标签触发按钮
// ----------------------------------------------------------

const props = defineProps<{
  value: string
  disabled?: boolean
}>()

const activeTab = inject<Ref<string>>('tabsActiveValue')!
const setActiveTab = inject<(value: string) => void>('tabsSetValue')!

const isActive = computed(() => activeTab.value === props.value)
</script>

<template>
  <button
    :disabled="disabled"
    class="inline-flex items-center justify-center gap-1.5 rounded-md px-3 py-1.5 text-sm
           font-medium transition-all focus-visible:outline-none focus-visible:ring-2
           focus-visible:ring-primary disabled:pointer-events-none disabled:opacity-50"
    :class="isActive ? 'bg-background text-foreground shadow-sm' : 'text-muted-foreground hover:text-foreground'"
    @click="setActiveTab(value)"
  >
    <slot />
  </button>
</template>
