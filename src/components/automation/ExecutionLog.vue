<script setup lang="ts">
import { computed } from 'vue'
import type { LogEntry, LogLevel } from '../../types'
import Button from '../ui/Button.vue'
import Badge from '../ui/Badge.vue'
import DynamicIcon from '../DynamicIcon.vue'

// ============================================================
// 执行日志面板
// 展示工作流运行时的日志条目，支持复制、导出和清空
// ============================================================

const props = defineProps<{
  entries: LogEntry[]
  selectedLogId: string | null
}>()

const emit = defineEmits<{
  clear: []
  export: []
  selectLog: [id: string]
}>()

// ---- 日志级别配置 ----
const levelConfig: Record<LogLevel, { icon: string; color: string; bgColor: string; label: string }> = {
  info:    { icon: 'info', color: 'text-[oklch(0.65_0.15_250)]', bgColor: 'bg-[oklch(0.65_0.15_250/0.15)]', label: '信息' },
  success: { icon: 'check-circle-2', color: 'text-[oklch(0.72_0.18_165)]', bgColor: 'bg-[oklch(0.72_0.18_165/0.15)]', label: '成功' },
  warning: { icon: 'alert-triangle', color: 'text-[oklch(0.7_0.18_85)]', bgColor: 'bg-[oklch(0.7_0.18_85/0.15)]', label: '警告' },
  error:   { icon: 'x-circle', color: 'text-destructive', bgColor: 'bg-destructive/15', label: '错误' },
}

// ---- 统计信息 ----
const stats = computed(() => ({
  total: props.entries.length,
  errors: props.entries.filter((e) => e.level === 'error').length,
  warnings: props.entries.filter((e) => e.level === 'warning').length,
}))

function formatTime(date: Date): string {
  return date.toLocaleTimeString('zh-CN', {
    hour: '2-digit', minute: '2-digit', second: '2-digit',
  }) + '.' + String(date.getMilliseconds()).padStart(3, '0')
}

function copyLogs() {
  const text = props.entries
    .map((e) => `[${formatTime(e.timestamp)}] ${e.level}: ${e.message}`)
    .join('\n')
  navigator.clipboard.writeText(text)
}
</script>

<template>
  <div class="flex h-full flex-col">
    <!-- 日志工具栏 -->
    <div class="flex items-center justify-between border-b border-border px-3 py-2">
      <div class="flex items-center gap-4">
        <span class="text-sm font-medium">执行日志</span>
        <div class="flex items-center gap-2 text-xs text-muted-foreground">
          <span>{{ stats.total }} 条记录</span>
          <span v-if="stats.errors > 0" class="text-destructive">{{ stats.errors }} 错误</span>
          <span v-if="stats.warnings > 0" class="text-[oklch(0.7_0.18_85)]">{{ stats.warnings }} 警告</span>
        </div>
      </div>
      <div class="flex items-center gap-1">
        <Button variant="ghost" size="sm" class="h-7 px-2 text-xs" @click="copyLogs">
          <DynamicIcon name="copy" :size="12" class="mr-1" />
          复制
        </Button>
        <Button variant="ghost" size="sm" class="h-7 px-2 text-xs" @click="emit('export')">
          <DynamicIcon name="download" :size="12" class="mr-1" />
          导出
        </Button>
        <Button variant="ghost" size="sm" class="h-7 px-2 text-xs text-muted-foreground hover:text-destructive" @click="emit('clear')">
          <DynamicIcon name="trash-2" :size="12" class="mr-1" />
          清空
        </Button>
      </div>
    </div>

    <!-- 日志条目列表 -->
    <div class="flex-1 overflow-auto">
      <div v-if="entries.length === 0" class="flex h-32 items-center justify-center text-sm text-muted-foreground">
        暂无日志记录
      </div>
      <div v-else class="divide-y divide-border/50">
        <div
          v-for="entry in entries"
          :key="entry.id"
          class="group flex items-start gap-3 px-3 py-2 font-mono text-xs cursor-pointer border-l-2 transition-colors"
          :class="[
            selectedLogId === entry.id ? 'bg-secondary border-l-primary' : 'border-l-transparent hover:bg-secondary/50',
            entry.level === 'error' ? 'bg-destructive/5' : ''
          ]"
          @click="emit('selectLog', entry.id)"
        >
          <!-- 时间戳 -->
          <span class="text-muted-foreground shrink-0 w-20">{{ formatTime(entry.timestamp) }}</span>

          <!-- 级别图标 -->
          <div class="shrink-0 rounded p-0.5" :class="levelConfig[entry.level].bgColor">
            <DynamicIcon :name="levelConfig[entry.level].icon" :size="12" :class="levelConfig[entry.level].color" />
          </div>

          <!-- 日志内容 -->
          <div class="flex-1 min-w-0">
            <div class="flex items-center gap-2">
              <span :class="levelConfig[entry.level].color" class="font-medium">{{ entry.message }}</span>
              <Badge v-if="entry.nodeName" variant="outline" class="text-[10px] px-1.5 py-0 font-normal">
                {{ entry.nodeName }}
              </Badge>
              <DynamicIcon v-if="entry.screenshot" name="image" :size="12" class="text-muted-foreground" />
            </div>
            <div v-if="entry.details" class="mt-1 text-muted-foreground truncate">{{ entry.details }}</div>
          </div>

          <!-- 选中指示箭头 -->
          <DynamicIcon name="chevron-right" :size="16" class="text-muted-foreground shrink-0 transition-opacity" :class="selectedLogId === entry.id ? 'opacity-100' : 'opacity-0 group-hover:opacity-100'" />
        </div>
      </div>
    </div>
  </div>
</template>
