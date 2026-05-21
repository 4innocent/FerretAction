<script setup lang="ts">
import { watch } from 'vue'
import { X } from 'lucide-vue-next'

// ----------------------------------------------------------
// 对话框组件
// 用法:
//   <Dialog v-model="isOpen">
//     <template #header>...</template>
//     <template #default>...</template>
//     <template #footer>...</template>
//   </Dialog>
// ----------------------------------------------------------

const props = withDefaults(defineProps<{
  modelValue: boolean
  maxWidth?: string
}>(), {
  maxWidth: 'max-w-lg',
})

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
}>()

function close() {
  emit('update:modelValue', false)
}

// 阻止背景滚动
watch(() => props.modelValue, (open) => {
  document.body.style.overflow = open ? 'hidden' : ''
})

function onBackdropClick(e: MouseEvent) {
  if ((e.target as HTMLElement).dataset.backdrop === 'true') {
    close()
  }
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') close()
}
</script>

<template>
  <Teleport to="body">
    <div
      v-if="modelValue"
      data-backdrop="true"
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm"
      @click="onBackdropClick"
      @keydown="onKeydown"
    >
      <div
        class="relative w-full rounded-lg border border-border bg-card shadow-lg animate-in"
        :class="maxWidth"
      >
        <!-- 关闭按钮 -->
        <button
          class="absolute right-4 top-4 rounded-sm opacity-70 transition-opacity hover:opacity-100
                 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary"
          @click="close"
        >
          <X :size="18" />
        </button>

        <slot />
      </div>
    </div>
  </Teleport>
</template>
