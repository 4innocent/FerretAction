<script setup lang="ts">
import { ref, computed } from 'vue'
import { ChevronDown } from 'lucide-vue-next'

// ----------------------------------------------------------
// 下拉选择组件
// 用法: <Select v-model="value" :options="[{value:'a',label:'A'}]" />
// ----------------------------------------------------------

const props = withDefaults(defineProps<{
  options: { value: string; label: string }[]
  placeholder?: string
  disabled?: boolean
}>(), {
  placeholder: '请选择...',
  disabled: false,
})

const model = defineModel<string>({ default: '' })

const isOpen = ref(false)
const selectRef = ref<HTMLElement>()

const selectedLabel = computed(() => {
  const opt = props.options.find((o) => o.value === model.value)
  return opt?.label ?? props.placeholder
})

function toggleOpen() {
  if (!props.disabled) {
    isOpen.value = !isOpen.value
  }
}

function selectOption(value: string) {
  model.value = value
  isOpen.value = false
}

function closeOnClickOutside(e: MouseEvent) {
  if (selectRef.value && !selectRef.value.contains(e.target as Node)) {
    isOpen.value = false
  }
}

import { onMounted, onUnmounted } from 'vue'

onMounted(() => document.addEventListener('click', closeOnClickOutside))
onUnmounted(() => document.removeEventListener('click', closeOnClickOutside))
</script>

<template>
  <div ref="selectRef" class="relative">
    <button
      type="button"
      :disabled="disabled"
      class="flex h-9 w-full items-center justify-between rounded-md border border-border
             bg-transparent px-3 py-1 text-sm shadow-sm transition-colors
             hover:bg-secondary/50 disabled:cursor-not-allowed disabled:opacity-50"
      @click="toggleOpen"
    >
      <span :class="{ 'text-muted-foreground': !model }">{{ selectedLabel }}</span>
      <ChevronDown :size="14" class="text-muted-foreground transition-transform" :class="{ 'rotate-180': isOpen }" />
    </button>

    <Teleport to="body">
      <div
        v-if="isOpen"
        class="absolute z-50 mt-1 max-h-60 w-full overflow-auto rounded-md border border-border
               bg-card p-1 shadow-md"
        :style="{ width: (selectRef?.offsetWidth ?? 200) + 'px', left: (selectRef?.getBoundingClientRect().left ?? 0) + 'px', top: (selectRef?.getBoundingClientRect().bottom ?? 0) + 4 + 'px' }"
      >
        <button
          v-for="opt in options"
          :key="opt.value"
          class="relative flex w-full cursor-pointer select-none items-center rounded-sm px-2 py-1.5
                 text-sm outline-none transition-colors hover:bg-secondary hover:text-foreground"
          :class="{ 'bg-primary/10 text-primary': opt.value === model }"
          @click="selectOption(opt.value)"
        >
          {{ opt.label }}
        </button>
      </div>
    </Teleport>
  </div>
</template>
