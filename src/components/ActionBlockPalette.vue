<template>
  <div class="action-palette">
    <div class="palette-section">
      <div class="section-title">控制流</div>
      <div class="blocks-grid">
        <div 
          v-for="block in controlBlocks" 
          :key="block.type"
          class="action-block"
          :style="{ '--block-color': block.color }"
          @mousedown="onDragStart($event, block)"
        >
          <div class="block-icon">
            <i :class="block.icon"></i>
          </div>
          <div class="block-content">
            <div class="block-label">{{ block.label }}</div>
            <div class="block-description">{{ block.description }}</div>
          </div>
        </div>
      </div>
    </div>

    <div class="palette-section">
      <div class="section-title">图像识别</div>
      <div class="blocks-grid">
        <div 
          v-for="block in imageBlocks" 
          :key="block.type"
          class="action-block"
          :style="{ '--block-color': block.color }"
          @mousedown="onDragStart($event, block)"
        >
          <div class="block-icon">
            <i :class="block.icon"></i>
          </div>
          <div class="block-content">
            <div class="block-label">{{ block.label }}</div>
            <div class="block-description">{{ block.description }}</div>
          </div>
        </div>
      </div>
    </div>

    <div class="palette-section">
      <div class="section-title">鼠标操作</div>
      <div class="blocks-grid">
        <div 
          v-for="block in mouseBlocks" 
          :key="block.type"
          class="action-block"
          :style="{ '--block-color': block.color }"
          @mousedown="onDragStart($event, block)"
        >
          <div class="block-icon">
            <i :class="block.icon"></i>
          </div>
          <div class="block-content">
            <div class="block-label">{{ block.label }}</div>
            <div class="block-description">{{ block.description }}</div>
          </div>
        </div>
      </div>
    </div>

    <div class="palette-section">
      <div class="section-title">键盘操作</div>
      <div class="blocks-grid">
        <div 
          v-for="block in keyboardBlocks" 
          :key="block.type"
          class="action-block"
          :style="{ '--block-color': block.color }"
          @mousedown="onDragStart($event, block)"
        >
          <div class="block-icon">
            <i :class="block.icon"></i>
          </div>
          <div class="block-content">
            <div class="block-label">{{ block.label }}</div>
            <div class="block-description">{{ block.description }}</div>
          </div>
        </div>
      </div>
    </div>

    <div class="palette-section">
      <div class="section-title">等待 & 延迟</div>
      <div class="blocks-grid">
        <div 
          v-for="block in waitBlocks" 
          :key="block.type"
          class="action-block"
          :style="{ '--block-color': block.color }"
          @mousedown="onDragStart($event, block)"
        >
          <div class="block-icon">
            <i :class="block.icon"></i>
          </div>
          <div class="block-content">
            <div class="block-label">{{ block.label }}</div>
            <div class="block-description">{{ block.description }}</div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import type { ActionBlock } from '../types'

const emit = defineEmits<{
  'drag-start': [block: ActionBlock, event: MouseEvent]
}>()

const controlBlocks = ref<ActionBlock[]>([
  { type: 'start', label: '开始', icon: 'pi pi-play', color: '#22c55e', description: '工作流的起点' },
  { type: 'end', label: '结束', icon: 'pi pi-stop-circle', color: '#22c55e', description: '工作流的终点' },
  { type: 'condition', label: '条件判断', icon: 'pi pi-question-circle', color: '#ec4899', description: '基于条件分支执行' },
  { type: 'loop', label: '循环', icon: 'pi pi-replay', color: '#ec4899', description: '重复执行一组动作' },
  { type: 'break', label: '跳出循环', icon: 'pi pi-sign-out', color: '#ec4899', description: '提前退出当前循环' },
])

const imageBlocks = ref<ActionBlock[]>([
  { type: 'find-image', label: '查找图像', icon: 'pi pi-search', color: '#6366f1', description: '在屏幕上定位图像目标' },
  { type: 'image-gone', label: '图像消失', icon: 'pi pi-eye-slash', color: '#6366f1', description: '等待图像消失' },
])

const mouseBlocks = ref<ActionBlock[]>([
  { type: 'move-mouse', label: '移动鼠标', icon: 'pi pi-arrows-alt', color: '#f59e0b', description: '移动鼠标到指定位置' },
  { type: 'click', label: '点击', icon: 'pi pi-external-link', color: '#f59e0b', description: '点击（按下后立即抬起）' },
  { type: 'double-click', label: '双击', icon: 'pi pi-clone', color: '#f59e0b', description: '执行双击操作' },
  { type: 'scroll', label: '滚动', icon: 'pi pi-sort-alt', color: '#f59e0b', description: '鼠标滚轮滚动' },
])

const keyboardBlocks = ref<ActionBlock[]>([
  { type: 'type-text', label: '输入文本', icon: 'pi pi-pencil', color: '#8b5cf6', description: '模拟键盘输入文本' },
  { type: 'hotkey', label: '快捷键', icon: 'pi pi-bolt', color: '#8b5cf6', description: '发送组合键' },
  { type: 'key-press', label: '按键', icon: 'pi pi-stop', color: '#8b5cf6', description: '按下单个按键' },
])

const waitBlocks = ref<ActionBlock[]>([
  { type: 'wait', label: '等待', icon: 'pi pi-clock', color: '#64748b', description: '等待指定时间' },
  { type: 'wait-condition', label: '等待条件', icon: 'pi pi-hourglass', color: '#475569', description: '等待条件满足' },
])

const onDragStart = (event: MouseEvent, block: ActionBlock) => {
  emit('drag-start', block, event)
}
</script>

<style scoped>
.action-palette {
  height: 100%;
  overflow-y: auto;
  padding: 12px;
}

.palette-section {
  margin-bottom: 20px;
}

.section-title {
  font-size: 0.6875rem;
  font-weight: 600;
  color: var(--text-color-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: 10px;
  padding-left: 4px;
}

.blocks-grid {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.action-block {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px;
  background: var(--surface-card);
  border: 1px solid var(--surface-border);
  border-radius: 8px;
  cursor: grab;
  transition: all 0.2s ease;
  border-left: 3px solid var(--block-color);
}

.action-block:hover {
  border-color: var(--block-color);
  background: var(--surface-hover);
  transform: translateX(4px);
}

.action-block:active {
  cursor: grabbing;
}

.block-icon {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: color-mix(in srgb, var(--block-color) 20%, transparent);
  border-radius: 6px;
  color: var(--block-color);
  flex-shrink: 0;
}

.block-content {
  flex: 1;
  min-width: 0;
}

.block-label {
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--text-color);
}

.block-description {
  font-size: 0.6875rem;
  color: var(--text-color-secondary);
  margin-top: 2px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
