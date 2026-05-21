<script setup lang="ts">
import { computed } from 'vue'

// ----------------------------------------------------------
// 通用按钮组件
// 支持 variant: default | ghost | outline
// 支持 size: default | sm | icon
// ----------------------------------------------------------

withDefaults(defineProps<{
  variant?: 'default' | 'ghost' | 'outline'
  size?: 'default' | 'sm' | 'icon'
  disabled?: boolean
}>(), {
  variant: 'default',
  size: 'default',
  disabled: false,
})

const variantClasses = computed(() => ({
  default: 'bg-primary text-primary-foreground hover:bg-primary/90',
  ghost: 'hover:bg-secondary hover:text-foreground',
  outline: 'border border-border bg-transparent hover:bg-secondary hover:text-foreground',
}))

const sizeClasses = computed(() => ({
  default: 'h-9 px-4 py-2',
  sm: 'h-8 px-3 text-xs',
  icon: 'h-8 w-8 p-0',
}))
</script>

<template>
  <button
    :disabled="disabled"
    class="inline-flex items-center justify-center gap-2 rounded-md text-sm font-medium
           transition-colors focus-visible:outline-none focus-visible:ring-2
           focus-visible:ring-primary disabled:pointer-events-none disabled:opacity-50"
    :class="[variantClasses[variant], sizeClasses[size]]"
  >
    <slot />
  </button>
</template>
