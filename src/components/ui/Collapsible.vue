<script setup lang="ts">
import { provide, ref, readonly } from 'vue'

// ----------------------------------------------------------
// 可折叠面板组件
// ----------------------------------------------------------

const props = withDefaults(defineProps<{
  open?: boolean
  defaultOpen?: boolean
}>(), {
  open: undefined as any,
  defaultOpen: true,
})

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const isOpen = ref(props.open ?? props.defaultOpen)

function toggle() {
  isOpen.value = !isOpen.value
  emit('update:open', isOpen.value)
}

provide('collapsibleOpen', readonly(isOpen))
provide('collapsibleToggle', toggle)
</script>

<template>
  <div>
    <slot />
  </div>
</template>
