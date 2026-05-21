<script setup lang="ts">
import type { ImageTargetData } from '../../types'
import ImageTarget from './ImageTarget.vue'

// ============================================================
// 图片目标列表容器
// ============================================================

defineProps<{
  targets: ImageTargetData[]
  selectedId: string | null
}>()

const emit = defineEmits<{
  select: [id: string]
  delete: [id: string]
  duplicate: [id: string]
}>()
</script>

<template>
  <div class="space-y-2">
    <ImageTarget
      v-for="target in targets"
      :key="target.id"
      :target="target"
      :isSelected="selectedId === target.id"
      @select="emit('select', target.id)"
      @delete="emit('delete', target.id)"
      @duplicate="emit('duplicate', target.id)"
    />
  </div>
</template>
