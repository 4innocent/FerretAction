<script setup lang="ts">
// ----------------------------------------------------------
// 开关组件
// ----------------------------------------------------------

const model = defineModel<boolean>({ required: true })

withDefaults(defineProps<{
  disabled?: boolean
}>(), {
  disabled: false,
})
</script>

<template>
  <button
    role="switch"
    :aria-checked="model"
    :disabled="disabled"
    class="relative inline-flex h-5 w-9 shrink-0 cursor-pointer items-center rounded-full
           transition-colors focus-visible:outline-none focus-visible:ring-2
           focus-visible:ring-primary disabled:cursor-not-allowed disabled:opacity-50"
    :class="model ? 'bg-primary' : 'bg-border'"
    @click="model = !model"
  >
    <span
      class="pointer-events-none block h-4 w-4 rounded-full bg-background shadow-lg ring-0
             transition-transform"
      :class="model ? 'translate-x-4' : 'translate-x-0.5'"
    />
  </button>
</template>
