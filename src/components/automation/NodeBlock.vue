<script setup lang="ts">
import type { NodeType } from '../../types'
import { NODE_TYPE_CONFIG } from '../../types'
import DynamicIcon from '../DynamicIcon.vue'

// ============================================================
// 侧边栏中可拖拽的节点积木块
// 用户拖拽到画布上即可创建对应类型的节点
// ============================================================

const props = defineProps<{
  type: NodeType
  label: string
  description: string
}>()

const emit = defineEmits<{
  dragStart: [e: DragEvent, type: NodeType]
}>()

const config = NODE_TYPE_CONFIG[props.type]

function onDragStart(e: DragEvent) {
  e.dataTransfer?.setData('nodeType', props.type)
  emit('dragStart', e, props.type)
}
</script>

<template>
  <div
    draggable="true"
    class="flex items-center gap-3 rounded-lg border border-border bg-card p-3
           cursor-grab transition-all hover:border-muted-foreground/50 hover:bg-secondary/50
           active:cursor-grabbing"
    @dragstart="onDragStart"
  >
    <div class="rounded-md p-2" :class="config.bgColor">
      <DynamicIcon :name="config.icon" :size="16" :class="config.color" />
    </div>
    <div class="flex-1 min-w-0">
      <div class="text-sm font-medium">{{ label }}</div>
      <div class="text-xs text-muted-foreground truncate">{{ description }}</div>
    </div>
  </div>
</template>
