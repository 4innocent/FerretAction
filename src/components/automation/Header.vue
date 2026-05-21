<script setup lang="ts">
import { computed } from 'vue'
import type { WorkflowData } from '../../types'
import Button from '../ui/Button.vue'
import Badge from '../ui/Badge.vue'
import Tooltip from '../ui/Tooltip.vue'
import {
  DropdownMenu, DropdownMenuTrigger, DropdownMenuContent,
  DropdownMenuItem, DropdownMenuSeparator,
} from '../ui'
import DynamicIcon from '../DynamicIcon.vue'

// ============================================================
// 顶部 Header 栏
// 包含 Logo、工作流选择器、运行控制按钮、文件菜单和设置入口
// ============================================================

const props = defineProps<{
  workflowName: string
  currentWorkflow: WorkflowData | null
  recentWorkflows: WorkflowData[]
  isRunning: boolean
  isPaused: boolean
  isDebugMode: boolean
  hasUnsavedChanges: boolean
}>()

const emit = defineEmits<{
  run: []
  pause: []
  stop: []
  stepForward: []
  toggleDebug: []
  save: []
  openSettings: []
  openWorkflowManager: []
  selectWorkflow: [id: string]
  newWorkflow: []
}>()

/** 工作流状态对应的图标和样式 */
const statusIcon = computed(() => {
  if (!props.currentWorkflow) return { icon: 'clock', class: 'text-muted-foreground' }
  switch (props.currentWorkflow.status) {
    case 'running': return { icon: 'zap', class: 'text-primary animate-pulse' }
    case 'success': return { icon: 'check-circle-2', class: 'text-[oklch(0.72_0.17_142)]' }
    case 'error': return { icon: 'alert-circle', class: 'text-destructive' }
    default: return { icon: 'clock', class: 'text-muted-foreground' }
  }
})
</script>

<template>
  <header class="flex h-14 items-center justify-between border-b border-border bg-card px-4 shrink-0">
    <!-- ====== 左侧：Logo + 工作流选择器 ====== -->
    <div class="flex items-center gap-4">
      <div class="flex items-center gap-2">
        <div class="flex h-8 w-8 items-center justify-center rounded-lg bg-primary">
          <DynamicIcon name="zap" :size="16" class="text-primary-foreground" />
        </div>
        <span class="font-semibold tracking-tight">PixelFlow</span>
      </div>

      <div class="h-6 w-px bg-border" />

      <!-- 工作流选择下拉 -->
      <DropdownMenu>
        <DropdownMenuTrigger>
          <Button variant="ghost" size="sm" class="h-9 gap-2 px-3">
            <DynamicIcon name="layers" :size="16" class="text-muted-foreground" />
            <span class="text-sm font-medium max-w-[160px] truncate">{{ workflowName }}</span>
            <Badge v-if="hasUnsavedChanges" variant="secondary" class="text-[10px] px-1.5 py-0">
              未保存
            </Badge>
            <DynamicIcon name="chevron-down" :size="12" class="text-muted-foreground" />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="start" class="w-72">
          <!-- 当前工作流标识 -->
          <template v-if="currentWorkflow">
            <div class="px-2 py-1.5">
              <div class="text-xs text-muted-foreground mb-1">当前工作流</div>
              <div class="flex items-center gap-2 rounded-md bg-primary/10 px-2 py-1.5">
                <DynamicIcon :name="statusIcon.icon" :size="12" :class="statusIcon.class" />
                <span class="text-sm font-medium flex-1 truncate">{{ currentWorkflow.name }}</span>
                <span class="text-xs text-muted-foreground">{{ currentWorkflow.nodeCount }} 节点</span>
              </div>
            </div>
            <DropdownMenuSeparator />
          </template>

          <!-- 最近使用的工作流 -->
          <template v-if="recentWorkflows.length > 0">
            <div class="px-2 py-1.5">
              <div class="text-xs text-muted-foreground">最近使用</div>
            </div>
            <DropdownMenuItem
              v-for="wf in recentWorkflows.slice(0, 5)"
              :key="wf.id"
              @click="emit('selectWorkflow', wf.id)"
              class="gap-2"
            >
              <DynamicIcon :name="wf.status === 'running' ? 'zap' : wf.status === 'success' ? 'check-circle-2' : 'clock'" :size="12" class="text-muted-foreground" />
              <span class="flex-1 truncate">{{ wf.name }}</span>
              <span class="text-xs text-muted-foreground">{{ wf.nodeCount }} 节点</span>
            </DropdownMenuItem>
            <DropdownMenuSeparator />
          </template>

          <DropdownMenuItem @click="emit('newWorkflow')">
            <DynamicIcon name="plus" :size="16" class="mr-2" />
            新建工作流
          </DropdownMenuItem>
          <DropdownMenuItem @click="emit('openWorkflowManager')">
            <DynamicIcon name="folder-open" :size="16" class="mr-2" />
            打开工作流管理器
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>

    <!-- ====== 中间：执行控制按钮 ====== -->
    <div class="flex items-center gap-1 rounded-lg border border-border bg-secondary/50 p-1">
      <!-- 运行 -->
      <Tooltip text="运行 (F5)">
        <Button
          :variant="isRunning && !isPaused ? 'default' : 'ghost'"
          size="icon"
          :disabled="isRunning && !isPaused"
          @click="emit('run')"
        >
          <DynamicIcon name="play" :size="16" />
        </Button>
      </Tooltip>

      <!-- 暂停 -->
      <Tooltip text="暂停 (F6)">
        <Button
          :variant="isPaused ? 'default' : 'ghost'"
          size="icon"
          :disabled="!isRunning"
          @click="emit('pause')"
        >
          <DynamicIcon name="pause" :size="16" />
        </Button>
      </Tooltip>

      <!-- 停止 -->
      <Tooltip text="停止 (F7)">
        <Button variant="ghost" size="icon" :disabled="!isRunning" @click="emit('stop')">
          <DynamicIcon name="square" :size="16" />
        </Button>
      </Tooltip>

      <div class="mx-1 h-4 w-px bg-border" />

      <!-- 单步执行 -->
      <Tooltip text="单步执行 (F10)">
        <Button variant="ghost" size="icon" :disabled="!isPaused" @click="emit('stepForward')">
          <DynamicIcon name="skip-forward" :size="16" />
        </Button>
      </Tooltip>

      <!-- 调试模式 -->
      <Tooltip text="调试模式">
        <Button
          :variant="isDebugMode ? 'default' : 'ghost'"
          size="icon"
          :class="isDebugMode ? 'text-[oklch(0.7_0.18_85)]' : ''"
          @click="emit('toggleDebug')"
        >
          <DynamicIcon name="bug" :size="16" />
        </Button>
      </Tooltip>
    </div>

    <!-- ====== 右侧：文件操作 + 设置 ====== -->
    <div class="flex items-center gap-2">
      <!-- 保存 -->
      <Tooltip text="保存 (Ctrl+S)">
        <Button variant="ghost" size="icon" @click="emit('save')">
          <DynamicIcon name="save" :size="16" />
        </Button>
      </Tooltip>

      <!-- 文件菜单 -->
      <DropdownMenu>
        <DropdownMenuTrigger>
          <Button variant="ghost" size="sm" class="h-8 px-2">
            <DynamicIcon name="folder-open" :size="16" class="mr-1" />
            文件
            <DynamicIcon name="chevron-down" :size="12" class="ml-1" />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end">
          <DropdownMenuItem @click="emit('newWorkflow')">
            <DynamicIcon name="plus" :size="16" class="mr-2" />
            新建
          </DropdownMenuItem>
          <DropdownMenuItem @click="emit('openWorkflowManager')">
            <DynamicIcon name="folder-open" :size="16" class="mr-2" />
            打开
          </DropdownMenuItem>
          <DropdownMenuItem @click="emit('save')">
            <DynamicIcon name="save" :size="16" class="mr-2" />
            保存
          </DropdownMenuItem>
          <DropdownMenuSeparator />
          <DropdownMenuItem>
            <DynamicIcon name="file-up" :size="16" class="mr-2" />
            导入
          </DropdownMenuItem>
          <DropdownMenuItem>
            <DynamicIcon name="file-down" :size="16" class="mr-2" />
            导出
          </DropdownMenuItem>
          <DropdownMenuSeparator />
          <DropdownMenuItem>
            <DynamicIcon name="history" :size="16" class="mr-2" />
            版本历史
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>

      <div class="h-6 w-px bg-border" />

      <Tooltip text="快捷键">
        <Button variant="ghost" size="icon">
          <DynamicIcon name="keyboard" :size="16" />
        </Button>
      </Tooltip>

      <Tooltip text="帮助">
        <Button variant="ghost" size="icon">
          <DynamicIcon name="help-circle" :size="16" />
        </Button>
      </Tooltip>

      <Tooltip text="设置">
        <Button variant="ghost" size="icon" @click="emit('openSettings')">
          <DynamicIcon name="settings" :size="16" />
        </Button>
      </Tooltip>
    </div>
  </header>
</template>
