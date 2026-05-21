<script setup lang="ts">
import { ref, computed } from 'vue'

// ----------------------------------------------------------
// 滑块组件
// ----------------------------------------------------------

const props = withDefaults(defineProps<{
  modelValue?: number[]
  min?: number
  max?: number
  step?: number
  disabled?: boolean
}>(), {
  modelValue: () => [50],
  min: 0,
  max: 100,
  step: 1,
  disabled: false,
})

const emit = defineEmits<{
  'update:modelValue': [value: number[]]
}>()

const trackRef = ref<HTMLElement>()

const percent = computed(() => {
  const val = props.modelValue[0] ?? props.min
  return ((val - props.min) / (props.max - props.min)) * 100
})

function setValueFromEvent(e: MouseEvent) {
  if (props.disabled || !trackRef.value) return
  const rect = trackRef.value.getBoundingClientRect()
  const pct = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width))
  const raw = props.min + pct * (props.max - props.min)
  const stepped = Math.round(raw / props.step) * props.step
  emit('update:modelValue', [stepped])
}
</script>

<template>
  <div
    ref="trackRef"
    role="slider"
    class="relative h-5 w-full cursor-pointer touch-none"
    :class="{ 'opacity-50 cursor-not-allowed': disabled }"
    @click="setValueFromEvent"
  >
    <div class="absolute top-1/2 h-1.5 w-full -translate-y-1/2 rounded-full bg-secondary">
      <div
        class="h-full rounded-full bg-primary transition-all"
        :style="{ width: percent + '%' }"
      />
    </div>
    <div
      class="absolute top-1/2 block h-4 w-4 -translate-y-1/2 rounded-full border-2
             border-primary bg-background shadow-sm transition-all"
      :style="{ left: `calc(${percent}% - 0.5rem)` }"
    />
  </div>
</template>
