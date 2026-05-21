<script setup lang="ts">
// ----------------------------------------------------------
// 多行文本输入组件
// ----------------------------------------------------------

const model = defineModel<string>({ required: true })

withDefaults(defineProps<{
  placeholder?: string
  disabled?: boolean
}>(), {
  disabled: false,
})
</script>

<template>
  <textarea
    v-model="model"
    :placeholder="placeholder"
    :disabled="disabled"
    class="flex min-h-[80px] w-full rounded-md border border-border bg-transparent px-3 py-2 text-sm
           shadow-sm transition-colors placeholder:text-muted-foreground
           focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-primary
           disabled:cursor-not-allowed disabled:opacity-50 resize-none"
  />
</template>
