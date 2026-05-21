<script setup lang="ts">
import { ref } from 'vue'
import type { LogEntry, MatchResult } from '../../types'
import Button from '../ui/Button.vue'
import { Tabs, TabsList, TabsTrigger, TabsContent } from '../ui'
import ExecutionLog from './ExecutionLog.vue'
import ScreenDebugger from './ScreenDebugger.vue'
import DynamicIcon from '../DynamicIcon.vue'

// ============================================================
// 底部面板
// 包含三个标签页：执行日志、屏幕调试、截图记录
// 支持折叠/展开
// ============================================================

const props = defineProps<{
  logs: LogEntry[]
  selectedLogId: string | null
  screenshot: string | null
  matchResults: MatchResult[]
  isSearching: boolean
  selectedMatchId: string | null
}>()

const emit = defineEmits<{
  clearLogs: []
  exportLogs: []
  selectLog: [id: string]
  capture: []
  search: []
  selectMatch: [id: string | null]
}>()

const isCollapsed = ref(false)
const panelHeight = ref(240)
const activeTab = ref('logs')

const errorCount = props.logs.filter((l) => l.level === 'error').length
</script>

<template>
  <div
    class="border-t border-border bg-card transition-all duration-200 shrink-0"
    :style="{ height: isCollapsed ? '40px' : panelHeight + 'px' }"
  >
    <Tabs v-model="activeTab">
      <!-- 标签页头部 -->
      <div class="flex items-center justify-between border-b border-border px-2">
        <TabsList class="h-9 bg-transparent p-0">
          <!-- 日志标签 -->
          <TabsTrigger value="logs" class="h-8 rounded-none border-b-2 border-transparent px-3 text-xs data-[state=active]:border-primary data-[state=active]:bg-transparent">
            <DynamicIcon name="terminal" :size="14" class="mr-1.5" />
            执行日志
            <span v-if="errorCount > 0" class="ml-1.5 rounded-full bg-destructive px-1.5 py-0.5 text-[10px] text-destructive-foreground">
              {{ errorCount }}
            </span>
          </TabsTrigger>

          <!-- 屏幕调试标签 -->
          <TabsTrigger value="debugger" class="h-8 rounded-none border-b-2 border-transparent px-3 text-xs data-[state=active]:border-primary data-[state=active]:bg-transparent">
            <DynamicIcon name="monitor" :size="14" class="mr-1.5" />
            屏幕调试
            <span v-if="matchResults.length > 0" class="ml-1.5 rounded-full bg-primary px-1.5 py-0.5 text-[10px] text-primary-foreground">
              {{ matchResults.length }}
            </span>
          </TabsTrigger>

          <!-- 截图记录标签 -->
          <TabsTrigger value="screenshots" class="h-8 rounded-none border-b-2 border-transparent px-3 text-xs data-[state=active]:border-primary data-[state=active]:bg-transparent">
            <DynamicIcon name="image" :size="14" class="mr-1.5" />
            截图记录
          </TabsTrigger>
        </TabsList>

        <!-- 折叠按钮 -->
        <Button variant="ghost" size="icon" class="h-7 w-7" @click="isCollapsed = !isCollapsed">
          <DynamicIcon :name="isCollapsed ? 'chevron-up' : 'chevron-down'" :size="16" />
        </Button>
      </div>

      <!-- 标签页内容 -->
      <div v-if="!isCollapsed" class="flex-1 min-h-0">
        <TabsContent value="logs">
          <ExecutionLog
            :entries="logs"
            :selectedLogId="selectedLogId"
            @clear="emit('clearLogs')"
            @export="emit('exportLogs')"
            @selectLog="emit('selectLog', $event)"
          />
        </TabsContent>

        <TabsContent value="debugger">
          <ScreenDebugger
            :screenshot="screenshot"
            :matchResults="matchResults"
            :searchRegion="null"
            :isSearching="isSearching"
            :selectedMatchId="selectedMatchId"
            @capture="emit('capture')"
            @search="emit('search')"
            @selectMatch="emit('selectMatch', $event)"
          />
        </TabsContent>

        <TabsContent value="screenshots">
          <div class="flex h-full items-center justify-center text-sm text-muted-foreground">
            <div class="text-center">
              <DynamicIcon name="image" :size="32" class="mx-auto text-muted-foreground/50" />
              <p class="mt-2">执行截图将显示在这里</p>
            </div>
          </div>
        </TabsContent>
      </div>
    </Tabs>
  </div>
</template>
