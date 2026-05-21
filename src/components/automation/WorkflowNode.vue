<script setup lang="ts">
import { computed } from 'vue'
import type { WorkflowNodeData, NodeStatus } from '../../types'
import { NODE_TYPE_CONFIG } from '../../types'
import { GripVertical, CheckCircle2, AlertCircle, Loader2 } from 'lucide-vue-next'
import DynamicIcon from '../DynamicIcon.vue'

// ============================================================
// 画布上的工作流节点卡片
// 显示节点类型图标、名称、状态以及输入/输出连接点
// ============================================================

const props = defineProps<{
  node: WorkflowNodeData
  isSelected: boolean
}>()

const emit = defineEmits<{
  select: []
  dragStart: [e: DragEvent]
}>()

const config = computed(() => NODE_TYPE_CONFIG[props.node.type])

/** 节点状态指示器 */
const statusComponent = computed(() => {
  const map: Record<NodeStatus, any> = {
    running: Loader2,
    success: CheckCircle2,
    error: AlertCircle,
    idle: null,
  }
  return map[props.node.status]
})

const statusClass = computed(() => {
  const map: Record<NodeStatus, string> = {
    running: 'text-primary animate-spin',
    success: 'text-[oklch(0.72_0.18_165)]',
    error: 'text-destructive',
    idle: 'hidden',
  }
  return map[props.node.status]
})

/** 节点主体摘要文本 */
const summary = computed(() => {
  switch (props.node.type) {
    case 'find': return '目标图片'
    case 'mouse': return '移动至 (0, 0)'
    case 'click': return '左键单击'
    case 'type': return '"文本内容"'
    case 'wait': return '等待 1000ms'
    case 'condition': return '如果找到图片...'
    case 'loop': return '重复 5 次'
    default: return ''
  }
})

function onDragStart(e: DragEvent) {
  e.dataTransfer?.setData('dragNodeId', props.node.id)
  emit('dragStart', e)
}
</script>

<template>
  <div
    class="absolute cursor-pointer select-none w-[180px] rounded-lg border bg-card
           transition-all duration-200"
    :class="[
      isSelected ? 'border-primary ring-2 ring-primary/30' : 'border-border hover:border-muted-foreground/50',
      node.status === 'running' ? 'border-primary/50' : '',
    ]"
    :style="{ left: node.x + 'px', top: node.y + 'px' }"
    draggable="true"
    @click.stop="emit('select')"
    @dragstart="onDragStart"
  >
    <!-- 节点头部：图标 + 名称 + 状态 -->
    <div class="flex items-center gap-2 rounded-t-lg px-3 py-2" :class="config.bgColor">
      <GripVertical :size="16" class="text-muted-foreground cursor-grab" />
      <DynamicIcon :name="config.icon" :size="16" :class="config.color" />
      <span class="flex-1 text-sm font-medium truncate">{{ node.label }}</span>
      <component
        v-if="statusComponent"
        :is="statusComponent"
        :size="14"
        :class="statusClass"
      />
    </div>

    <!-- 节点主体：类型摘要 -->
    <div class="px-3 py-2 text-xs text-muted-foreground">
      <template v-if="node.type === 'find'">
        <div class="flex items-center gap-2">
          <div class="h-8 w-8 rounded border border-border bg-secondary" />
          <span>{{ summary }}</span>
        </div>
      </template>
      <template v-else-if="node.type === 'type'">
        <span class="font-mono">{{ summary }}</span>
      </template>
      <template v-else>
        <span>{{ summary }}</span>
      </template>
    </div>

    <!-- 连接点：上输入端口 -->
    <div class="absolute -top-1.5 left-1/2 -translate-x-1/2 h-3 w-3 rounded-full border-2 border-border bg-card" />
    <!-- 连接点：下输出端口 -->
    <div class="absolute -bottom-1.5 left-1/2 -translate-x-1/2 h-3 w-3 rounded-full border-2 border-border bg-card" />
  </div>
</template>
