<template>
  <div class="execution-logs">
    <div class="logs-toolbar">
      <div class="filter-group">
        <Button 
          v-for="level in logLevels" 
          :key="level.value"
          :label="level.label"
          :icon="level.icon"
          :severity="activeFilter === level.value ? undefined : 'secondary'"
          :outlined="activeFilter !== level.value"
          size="small"
          @click="activeFilter = level.value"
        />
      </div>
      <div class="toolbar-actions">
        <Button icon="pi pi-trash" v-tooltip="'清空日志'" text size="small" severity="danger" @click="$emit('clear')" />
      </div>
    </div>
    
    <div class="logs-container" ref="logsContainer">
      <div 
        v-for="log in filteredLogs" 
        :key="log.id"
        class="log-entry"
        :class="[`log-${log.level}`, { highlighted: highlightedLog === log.id }]"
        @click="highlightLog(log)"
      >
        <div class="log-time">{{ formatTime(log.timestamp) }}</div>
        <div class="log-icon">
          <i :class="getLogIcon(log.level)"></i>
        </div>
        <div class="log-message">{{ log.message }}</div>
        <div class="log-node" v-if="log.nodeId">
          <span class="node-tag">{{ log.nodeId }}</span>
        </div>
      </div>
      
      <div v-if="filteredLogs.length === 0" class="empty-logs">
        <i class="pi pi-inbox"></i>
        <span>暂无日志记录</span>
      </div>
    </div>
    
    <div class="logs-footer">
      <div class="log-stats">
        <span class="stat stat-info">
          <i class="pi pi-info-circle"></i> {{ infoCount }}
        </span>
        <span class="stat stat-success">
          <i class="pi pi-check-circle"></i> {{ successCount }}
        </span>
        <span class="stat stat-warning">
          <i class="pi pi-exclamation-triangle"></i> {{ warningCount }}
        </span>
        <span class="stat stat-error">
          <i class="pi pi-times-circle"></i> {{ errorCount }}
        </span>
      </div>
      <div class="auto-scroll">
        <Checkbox v-model="autoScroll" :binary="true" inputId="autoScroll" />
        <label for="autoScroll">自动滚动</label>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue'
import Button from 'primevue/button'
import Checkbox from 'primevue/checkbox'
import type { ExecutionLog } from '../types'

const props = defineProps<{
  logs: ExecutionLog[]
}>()

const emit = defineEmits<{
  clear: []
}>()

const logsContainer = ref<HTMLElement>()
const activeFilter = ref('all')
const autoScroll = ref(true)
const highlightedLog = ref<string | null>(null)

const logLevels = [
  { value: 'all', label: '全部', icon: 'pi pi-list' },
  { value: 'info', label: '信息', icon: 'pi pi-info-circle' },
  { value: 'success', label: '成功', icon: 'pi pi-check-circle' },
  { value: 'warning', label: '警告', icon: 'pi pi-exclamation-triangle' },
  { value: 'error', label: '错误', icon: 'pi pi-times-circle' }
]

const filteredLogs = computed(() => {
  if (activeFilter.value === 'all') return props.logs
  return props.logs.filter(log => log.level === activeFilter.value)
})

const infoCount = computed(() => props.logs.filter(l => l.level === 'info').length)
const successCount = computed(() => props.logs.filter(l => l.level === 'success').length)
const warningCount = computed(() => props.logs.filter(l => l.level === 'warning').length)
const errorCount = computed(() => props.logs.filter(l => l.level === 'error').length)

const formatTime = (date: Date): string => {
  return date.toLocaleTimeString('zh-CN', {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    hour12: false
  })
}

const getLogIcon = (level: string): string => {
  const icons: Record<string, string> = {
    info: 'pi pi-info-circle',
    success: 'pi pi-check-circle',
    warning: 'pi pi-exclamation-triangle',
    error: 'pi pi-times-circle'
  }
  return icons[level] || 'pi pi-circle'
}

const highlightLog = (log: ExecutionLog) => {
  highlightedLog.value = log.id
  setTimeout(() => {
    highlightedLog.value = null
  }, 1000)
}

// Auto-scroll on new logs
watch(() => props.logs.length, async () => {
  if (autoScroll.value && logsContainer.value) {
    await nextTick()
    logsContainer.value.scrollTop = logsContainer.value.scrollHeight
  }
})
</script>

<style scoped>
.execution-logs {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.logs-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 16px;
  border-bottom: 1px solid var(--surface-border);
}

.filter-group {
  display: flex;
  gap: 4px;
}

.filter-group :deep(.p-button) {
  font-size: 0.6875rem;
  padding: 4px 8px;
}

.toolbar-actions {
  display: flex;
  gap: 4px;
}

.logs-container {
  flex: 1;
  overflow-y: auto;
  padding: 8px 0;
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  font-size: 0.75rem;
}

.log-entry {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 6px 16px;
  transition: background 0.2s;
  cursor: pointer;
}

.log-entry:hover {
  background: var(--surface-hover);
}

.log-entry.highlighted {
  background: rgba(99, 102, 241, 0.1);
}

.log-time {
  color: var(--text-color-secondary);
  font-size: 0.6875rem;
  flex-shrink: 0;
  width: 70px;
}

.log-icon {
  flex-shrink: 0;
  width: 20px;
}

.log-info .log-icon {
  color: var(--cyan-400);
}

.log-success .log-icon {
  color: var(--green-400);
}

.log-warning .log-icon {
  color: var(--amber-400);
}

.log-error .log-icon {
  color: var(--red-400);
}

.log-message {
  flex: 1;
  color: var(--text-color);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.log-info .log-message {
  color: var(--text-color-secondary);
}

.log-error .log-message {
  color: var(--red-400);
}

.log-node {
  flex-shrink: 0;
}

.node-tag {
  padding: 2px 8px;
  background: var(--surface-card);
  border: 1px solid var(--surface-border);
  border-radius: 4px;
  font-size: 0.625rem;
  color: var(--text-color-secondary);
}

.empty-logs {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--text-color-secondary);
  gap: 8px;
}

.empty-logs i {
  font-size: 1.5rem;
  opacity: 0.5;
}

.logs-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 16px;
  border-top: 1px solid var(--surface-border);
  font-size: 0.6875rem;
}

.log-stats {
  display: flex;
  gap: 16px;
}

.stat {
  display: flex;
  align-items: center;
  gap: 4px;
}

.stat-info { color: var(--cyan-400); }
.stat-success { color: var(--green-400); }
.stat-warning { color: var(--amber-400); }
.stat-error { color: var(--red-400); }

.auto-scroll {
  display: flex;
  align-items: center;
  gap: 6px;
  color: var(--text-color-secondary);
}

.auto-scroll label {
  cursor: pointer;
}
</style>
