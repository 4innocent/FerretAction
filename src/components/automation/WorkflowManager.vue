<script setup lang="ts">
import { ref, computed } from 'vue'
import type { WorkflowData, FolderData, WorkflowStatus } from '../../types'
import Button from '../ui/Button.vue'
import Input from '../ui/Input.vue'
import Badge from '../ui/Badge.vue'
import Select from '../ui/Select.vue'
import {
  DropdownMenu, DropdownMenuTrigger, DropdownMenuContent,
  DropdownMenuItem, DropdownMenuSeparator,
} from '../ui'
import Tooltip from '../ui/Tooltip.vue'
import DynamicIcon from '../DynamicIcon.vue'

// ============================================================
// 工作流管理器弹窗
// 支持搜索、筛选、收藏、文件夹管理、重命名、复制、删除等操作
// ============================================================

const props = defineProps<{
  workflows: WorkflowData[]
  folders: FolderData[]
  currentWorkflowId: string
  isOpen: boolean
}>()

const emit = defineEmits<{
  close: []
  selectWorkflow: [id: string]
  createWorkflow: [name: string, folderId?: string]
  deleteWorkflow: [id: string]
  duplicateWorkflow: [id: string]
  renameWorkflow: [id: string, newName: string]
  runWorkflow: [id: string]
  toggleFavorite: [id: string]
  createFolder: [name: string]
  deleteFolder: [id: string]
  renameFolder: [id: string, newName: string]
  toggleFolderExpand: [id: string]
  moveToFolder: [workflowId: string, folderId: string | null]
  importWorkflow: []
  exportWorkflow: [id: string]
}>()

// ---- 内部状态 ----
const searchQuery = ref('')
const filterStatus = ref('all')
const isCreatingWorkflow = ref(false)
const isCreatingFolder = ref(false)
const newWorkflowName = ref('')
const newFolderName = ref('')
const selectedFolderId = ref<string | null>(null)
const editingId = ref<string | null>(null)
const editingName = ref('')

// ---- 筛选后的工作流列表 ----
const filteredWorkflows = computed(() =>
  props.workflows.filter((w) => {
    const matchesSearch = w.name.toLowerCase().includes(searchQuery.value.toLowerCase())
    const matchesStatus = filterStatus.value === 'all' || w.status === filterStatus.value
    return matchesSearch && matchesStatus
  })
)

// ---- 分组 ----
const favoriteWorkflows = computed(() => filteredWorkflows.value.filter((w) => w.isFavorite && !w.folderId))
const unfolderedWorkflows = computed(() => filteredWorkflows.value.filter((w) => !w.isFavorite && !w.folderId))

function getWorkflowsInFolder(folderId: string) {
  return filteredWorkflows.value.filter((w) => w.folderId === folderId)
}

// ---- 时间格式化 ----
function formatDate(date: Date): string {
  const now = new Date()
  const diff = now.getTime() - date.getTime()
  const minutes = Math.floor(diff / 60000)
  const hours = Math.floor(diff / 3600000)
  const days = Math.floor(diff / 86400000)
  if (minutes < 1) return '刚刚'
  if (minutes < 60) return `${minutes} 分钟前`
  if (hours < 24) return `${hours} 小时前`
  if (days < 7) return `${days} 天前`
  return date.toLocaleDateString('zh-CN')
}

// ---- 状态图标映射 ----
const statusIconMap: Record<WorkflowStatus, string> = {
  running: 'zap',
  success: 'check-circle-2',
  error: 'alert-circle',
  idle: 'clock',
}

const statusIconClass: Record<WorkflowStatus, string> = {
  running: 'text-primary animate-pulse',
  success: 'text-[oklch(0.72_0.17_142)]',
  error: 'text-destructive',
  idle: 'text-muted-foreground',
}

// ---- 筛选选项 ----
const filterOptions = [
  { value: 'all', label: '全部状态' },
  { value: 'idle', label: '空闲' },
  { value: 'running', label: '运行中' },
  { value: 'success', label: '成功' },
  { value: 'error', label: '错误' },
]

// ---- 创建/重命名处理 ----
function handleCreateWorkflow() {
  if (!newWorkflowName.value.trim()) return
  emit('createWorkflow', newWorkflowName.value.trim(), selectedFolderId.value || undefined)
  newWorkflowName.value = ''
  selectedFolderId.value = null
  isCreatingWorkflow.value = false
}

function handleCreateFolder() {
  if (!newFolderName.value.trim()) return
  emit('createFolder', newFolderName.value.trim())
  newFolderName.value = ''
  isCreatingFolder.value = false
}

function handleRename(id: string, isFolder: boolean) {
  if (!editingName.value.trim()) {
    editingId.value = null
    editingName.value = ''
    return
  }
  if (isFolder) emit('renameFolder', id, editingName.value.trim())
  else emit('renameWorkflow', id, editingName.value.trim())
  editingId.value = null
  editingName.value = ''
}
</script>

<template>
  <!-- 遮罩层 -->
  <Teleport to="body">
    <div
      v-if="isOpen"
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm"
      @click.self="emit('close')"
      @keydown.escape="emit('close')"
    >
      <div class="relative w-full max-w-2xl max-h-[80vh] flex flex-col rounded-lg border border-border bg-card shadow-lg">
        <!-- ====== 头部 ====== -->
        <div class="flex items-center justify-between px-6 py-4 border-b border-border">
          <div class="flex items-center gap-3">
            <div class="flex h-10 w-10 items-center justify-center rounded-lg bg-primary">
              <DynamicIcon name="layers" :size="20" class="text-primary-foreground" />
            </div>
            <div>
              <h2 class="text-lg font-semibold">工作流管理</h2>
              <p class="text-xs text-muted-foreground">管理所有自动化工作流</p>
            </div>
          </div>
          <Badge variant="outline">{{ workflows.length }} 个工作流</Badge>
        </div>

        <!-- ====== 工具栏 ====== -->
        <div class="flex items-center gap-2 px-6 py-3 border-b border-border">
          <div class="relative flex-1">
            <DynamicIcon name="search" :size="16" class="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground" />
            <Input v-model="searchQuery" placeholder="搜索工作流..." class="pl-9 h-9" />
          </div>
          <div class="w-32">
            <Select v-model="filterStatus" :options="filterOptions" />
          </div>
          <Button variant="outline" size="sm" class="h-9" @click="isCreatingFolder = true">
            <DynamicIcon name="folder-plus" :size="16" class="mr-1" />
            新建文件夹
          </Button>
          <Button variant="outline" size="sm" class="h-9" @click="emit('importWorkflow')">
            <DynamicIcon name="file-up" :size="16" class="mr-1" />
            导入
          </Button>
          <Button size="sm" class="h-9" @click="isCreatingWorkflow = true">
            <DynamicIcon name="plus" :size="16" class="mr-1" />
            新建工作流
          </Button>
        </div>

        <!-- ====== 内容区 ====== -->
        <div class="flex-1 overflow-auto px-6 py-4">
          <!-- 收藏区 -->
          <div v-if="favoriteWorkflows.length > 0" class="mb-6">
            <div class="flex items-center gap-2 mb-3">
              <DynamicIcon name="star" :size="16" class="text-[oklch(0.8_0.15_85)]" />
              <span class="text-sm font-medium text-muted-foreground">收藏</span>
            </div>
            <div class="space-y-2">
              <div
                v-for="wf in favoriteWorkflows"
                :key="wf.id"
                class="group relative flex items-center gap-3 rounded-lg border p-3 transition-all cursor-pointer"
                :class="currentWorkflowId === wf.id ? 'border-primary bg-primary/10' : 'border-transparent hover:border-border hover:bg-secondary/50'"
                @click="emit('selectWorkflow', wf.id); emit('close')"
              >
                <!-- 状态图标 -->
                <div class="flex h-9 w-9 items-center justify-center rounded-lg bg-secondary">
                  <DynamicIcon :name="statusIconMap[wf.status]" :size="14" :class="statusIconClass[wf.status]" />
                </div>

                <div class="flex-1 min-w-0">
                  <!-- 编辑态或显示态 -->
                  <Input
                    v-if="editingId === wf.id"
                    v-model="editingName"
                    class="h-6 px-1 text-sm"
                    @blur="handleRename(wf.id, false)"
                    @keydown.enter="handleRename(wf.id, false)"
                    @keydown.escape="editingId = null; editingName = ''"
                    @click.stop
                  />
                  <div v-else class="flex items-center gap-2">
                    <span class="font-medium text-sm truncate">{{ wf.name }}</span>
                    <DynamicIcon v-if="wf.isFavorite" name="star" :size="12" class="text-[oklch(0.8_0.15_85)]" />
                  </div>
                  <div class="flex items-center gap-2 mt-0.5">
                    <span class="text-xs text-muted-foreground">{{ wf.nodeCount }} 节点</span>
                    <span class="text-xs text-muted-foreground">·</span>
                    <span class="text-xs text-muted-foreground">{{ formatDate(wf.lastModified) }}</span>
                  </div>
                </div>

                <!-- 操作按钮 -->
                <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                  <Tooltip :text="wf.isFavorite ? '取消收藏' : '收藏'">
                    <Button variant="ghost" size="icon" class="h-7 w-7" @click.stop="emit('toggleFavorite', wf.id)">
                      <DynamicIcon :name="wf.isFavorite ? 'star-off' : 'star'" :size="14" />
                    </Button>
                  </Tooltip>
                  <Tooltip text="运行">
                    <Button variant="ghost" size="icon" class="h-7 w-7" @click.stop="emit('runWorkflow', wf.id)">
                      <DynamicIcon name="play" :size="14" />
                    </Button>
                  </Tooltip>
                  <DropdownMenu>
                    <DropdownMenuTrigger>
                      <Button variant="ghost" size="icon" class="h-7 w-7" @click.stop>
                        <DynamicIcon name="more-horizontal" :size="14" />
                      </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="end">
                      <DropdownMenuItem @click="editingId = wf.id; editingName = wf.name">
                        <DynamicIcon name="edit-3" :size="16" class="mr-2" /> 重命名
                      </DropdownMenuItem>
                      <DropdownMenuItem @click="emit('duplicateWorkflow', wf.id)">
                        <DynamicIcon name="copy" :size="16" class="mr-2" /> 复制
                      </DropdownMenuItem>
                      <DropdownMenuItem @click="emit('exportWorkflow', wf.id)">
                        <DynamicIcon name="file-down" :size="16" class="mr-2" /> 导出
                      </DropdownMenuItem>
                      <DropdownMenuSeparator />
                      <DropdownMenuItem class="text-destructive" @click="emit('deleteWorkflow', wf.id)">
                        <DynamicIcon name="trash-2" :size="16" class="mr-2" /> 删除
                      </DropdownMenuItem>
                    </DropdownMenuContent>
                  </DropdownMenu>
                </div>
              </div>
            </div>
          </div>

          <!-- 文件夹区 -->
          <div v-if="folders.length > 0" class="mb-6">
            <div class="flex items-center gap-2 mb-3">
              <DynamicIcon name="folder-open" :size="16" class="text-muted-foreground" />
              <span class="text-sm font-medium text-muted-foreground">文件夹</span>
            </div>
            <div v-for="folder in folders" :key="folder.id" class="mb-1">
              <!-- 文件夹标题行 -->
              <div class="group flex items-center gap-2 rounded-lg px-2 py-1.5 hover:bg-secondary/50 cursor-pointer" @click="emit('toggleFolderExpand', folder.id)">
                <DynamicIcon :name="folder.isExpanded ? 'chevron-down' : 'chevron-right'" :size="16" class="text-muted-foreground" />
                <DynamicIcon name="folder-open" :size="16" class="text-[oklch(0.8_0.15_85)]" />

                <Input
                  v-if="editingId === folder.id"
                  v-model="editingName"
                  class="h-5 px-1 text-sm flex-1"
                  @blur="handleRename(folder.id, true)"
                  @keydown.enter="handleRename(folder.id, true)"
                  @keydown.escape="editingId = null; editingName = ''"
                  @click.stop
                />
                <span v-else class="flex-1 text-sm font-medium">{{ folder.name }}</span>

                <Badge variant="secondary" class="text-[10px] px-1.5 py-0">
                  {{ getWorkflowsInFolder(folder.id).length }}
                </Badge>

                <DropdownMenu>
                  <DropdownMenuTrigger>
                    <Button variant="ghost" size="icon" class="h-6 w-6 opacity-0 group-hover:opacity-100" @click.stop>
                      <DynamicIcon name="more-horizontal" :size="14" />
                    </Button>
                  </DropdownMenuTrigger>
                  <DropdownMenuContent align="end">
                    <DropdownMenuItem @click="editingId = folder.id; editingName = folder.name">
                      <DynamicIcon name="edit-3" :size="16" class="mr-2" /> 重命名
                    </DropdownMenuItem>
                    <DropdownMenuSeparator />
                    <DropdownMenuItem class="text-destructive" @click="emit('deleteFolder', folder.id)">
                      <DynamicIcon name="trash-2" :size="16" class="mr-2" /> 删除文件夹
                    </DropdownMenuItem>
                  </DropdownMenuContent>
                </DropdownMenu>
              </div>

              <!-- 文件夹内的工作流 -->
              <div v-if="folder.isExpanded" class="ml-6 space-y-1 mt-1">
                <template v-if="getWorkflowsInFolder(folder.id).length > 0">
                  <div
                    v-for="wf in getWorkflowsInFolder(folder.id)"
                    :key="wf.id"
                    class="group relative flex items-center gap-3 rounded-lg border p-3 transition-all cursor-pointer"
                    :class="currentWorkflowId === wf.id ? 'border-primary bg-primary/10' : 'border-transparent hover:border-border hover:bg-secondary/50'"
                    @click="emit('selectWorkflow', wf.id); emit('close')"
                  >
                    <div class="flex h-9 w-9 items-center justify-center rounded-lg bg-secondary">
                      <DynamicIcon :name="statusIconMap[wf.status]" :size="14" :class="statusIconClass[wf.status]" />
                    </div>
                    <div class="flex-1 min-w-0">
                      <div class="flex items-center gap-2">
                        <span class="font-medium text-sm truncate">{{ wf.name }}</span>
                      </div>
                      <div class="flex items-center gap-2 mt-0.5">
                        <span class="text-xs text-muted-foreground">{{ wf.nodeCount }} 节点</span>
                        <span class="text-xs text-muted-foreground">·</span>
                        <span class="text-xs text-muted-foreground">{{ formatDate(wf.lastModified) }}</span>
                      </div>
                    </div>
                  </div>
                </template>
                <div v-else class="py-4 text-center text-sm text-muted-foreground">文件夹为空</div>
              </div>
            </div>
          </div>

          <!-- 无文件夹的工作流 -->
          <div v-if="unfolderedWorkflows.length > 0">
            <div class="flex items-center gap-2 mb-3">
              <DynamicIcon name="layers" :size="16" class="text-muted-foreground" />
              <span class="text-sm font-medium text-muted-foreground">全部工作流</span>
            </div>
            <div class="space-y-2">
              <div
                v-for="wf in unfolderedWorkflows"
                :key="wf.id"
                class="group relative flex items-center gap-3 rounded-lg border p-3 transition-all cursor-pointer"
                :class="currentWorkflowId === wf.id ? 'border-primary bg-primary/10' : 'border-transparent hover:border-border hover:bg-secondary/50'"
                @click="emit('selectWorkflow', wf.id); emit('close')"
              >
                <div class="flex h-9 w-9 items-center justify-center rounded-lg bg-secondary">
                  <DynamicIcon :name="statusIconMap[wf.status]" :size="14" :class="statusIconClass[wf.status]" />
                </div>
                <div class="flex-1 min-w-0">
                  <div class="flex items-center gap-2">
                    <span class="font-medium text-sm truncate">{{ wf.name }}</span>
                  </div>
                  <div class="flex items-center gap-2 mt-0.5">
                    <span class="text-xs text-muted-foreground">{{ wf.nodeCount }} 节点</span>
                    <span class="text-xs text-muted-foreground">·</span>
                    <span class="text-xs text-muted-foreground">{{ formatDate(wf.lastModified) }}</span>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- 空状态 -->
          <div v-if="filteredWorkflows.length === 0" class="flex flex-col items-center justify-center py-12">
            <div class="flex h-16 w-16 items-center justify-center rounded-full bg-secondary mb-4">
              <DynamicIcon name="layers" :size="32" class="text-muted-foreground" />
            </div>
            <p class="text-muted-foreground mb-4">
              {{ searchQuery ? '未找到匹配的工作流' : '还没有工作流' }}
            </p>
            <Button @click="isCreatingWorkflow = true">
              <DynamicIcon name="plus" :size="16" class="mr-2" />
              创建第一个工作流
            </Button>
          </div>
        </div>
      </div>

      <!-- ====== 新建工作流子对话框 ====== -->
      <Teleport to="body">
        <div
          v-if="isCreatingWorkflow"
          class="fixed inset-0 z-[60] flex items-center justify-center bg-black/30"
          @click.self="isCreatingWorkflow = false"
        >
          <div class="w-full max-w-md rounded-lg border border-border bg-card p-6 shadow-lg">
            <h3 class="text-lg font-semibold mb-1">新建工作流</h3>
            <p class="text-sm text-muted-foreground mb-4">创建一个新的自动化工作流</p>
            <div class="space-y-4">
              <div class="space-y-2">
                <label class="text-sm font-medium">工作流名称</label>
                <Input v-model="newWorkflowName" placeholder="输入工作流名称..." @keydown.enter="handleCreateWorkflow" />
              </div>
              <div v-if="folders.length > 0" class="space-y-2">
                <label class="text-sm font-medium">文件夹 (可选)</label>
                <Select :modelValue="selectedFolderId || 'none'" @update:modelValue="(v: string) => selectedFolderId = v === 'none' ? null : v" :options="[{ value: 'none', label: '无文件夹' }, ...folders.map(f => ({ value: f.id, label: f.name }))]" />
              </div>
            </div>
            <div class="flex justify-end gap-2 mt-6">
              <Button variant="outline" @click="isCreatingWorkflow = false">取消</Button>
              <Button :disabled="!newWorkflowName.trim()" @click="handleCreateWorkflow">创建</Button>
            </div>
          </div>
        </div>
      </Teleport>

      <!-- ====== 新建文件夹子对话框 ====== -->
      <Teleport to="body">
        <div
          v-if="isCreatingFolder"
          class="fixed inset-0 z-[60] flex items-center justify-center bg-black/30"
          @click.self="isCreatingFolder = false"
        >
          <div class="w-full max-w-md rounded-lg border border-border bg-card p-6 shadow-lg">
            <h3 class="text-lg font-semibold mb-1">新建文件夹</h3>
            <p class="text-sm text-muted-foreground mb-4">创建一个文件夹来组织工作流</p>
            <div class="space-y-4">
              <div class="space-y-2">
                <label class="text-sm font-medium">文件夹名称</label>
                <Input v-model="newFolderName" placeholder="输入文件夹名称..." @keydown.enter="handleCreateFolder" />
              </div>
            </div>
            <div class="flex justify-end gap-2 mt-6">
              <Button variant="outline" @click="isCreatingFolder = false">取消</Button>
              <Button :disabled="!newFolderName.trim()" @click="handleCreateFolder">创建</Button>
            </div>
          </div>
        </div>
      </Teleport>
    </div>
  </Teleport>
</template>
