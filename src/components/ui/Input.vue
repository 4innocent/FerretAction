<script setup lang="ts">
// ----------------------------------------------------------
// 输入框组件
// ----------------------------------------------------------

const model = defineModel<string>({ required: true })

withDefaults(defineProps<{
  placeholder?: string
  type?: string
  disabled?: boolean
}>(), {
  type: 'text',
  disabled: false,
})
</script>

<template>
  <input
    v-model="model"
    :type="type"
    :placeholder="placeholder"
    :disabled="disabled"
    class="flex h-9 w-full rounded-md border border-border bg-transparent px-3 py-1 text-sm
           shadow-sm transition-colors placeholder:text-muted-foreground
           focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-primary
           disabled:cursor-not-allowed disabled:opacity-50 font-mono"
  />
</template>
