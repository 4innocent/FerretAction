<script setup lang="ts">
import { computed } from 'vue'

// ----------------------------------------------------------
// 徽标组件
// ----------------------------------------------------------

withDefaults(defineProps<{
  variant?: 'default' | 'secondary' | 'outline' | 'destructive'
}>(), {
  variant: 'default',
})

const variantClasses = computed(() => ({
  default: 'bg-primary text-primary-foreground',
  secondary: 'bg-secondary text-secondary-foreground',
  outline: 'border border-border text-foreground',
  destructive: 'bg-destructive text-destructive-foreground',
}))
</script>

<template>
  <span
    class="inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium"
    :class="variantClasses[variant]"
  >
    <slot />
  </span>
</template>
