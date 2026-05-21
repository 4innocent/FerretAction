import { ref, computed } from 'vue'
import type {
  WorkflowNodeData,
  WorkflowData,
  FolderData,
  ImageTargetData,
  LogEntry,
  MatchResult,
  Connection,
  NodeType,
} from '../types'

// ============================================================
// 自动化工作流 - 全局状态管理
// 所有 demo 数据、业务逻辑和状态变更都在此 composable 中
// ============================================================

// ----------------------------------------------------------
// Demo 初始数据
// ----------------------------------------------------------

function createInitialWorkflows(): WorkflowData[] {
  return [
    {
      id: 'workflow-1', name: 'Login Automation', description: '自动登录网站并验证登录状态',
      lastModified: new Date(Date.now() - 300000), lastRun: new Date(Date.now() - 120000),
      status: 'running', nodeCount: 7, isFavorite: true, tags: ['登录', '认证'],
    },
    {
      id: 'workflow-2', name: '数据采集流程', description: '从多个页面采集数据并保存',
      folderId: 'folder-1', lastModified: new Date(Date.now() - 3600000),
      lastRun: new Date(Date.now() - 1800000), status: 'success', nodeCount: 12,
      isFavorite: false, tags: ['采集', '数据'],
    },
    {
      id: 'workflow-3', name: '表单自动填充', description: '自动填写多个表单字段',
      folderId: 'folder-1', lastModified: new Date(Date.now() - 7200000),
      status: 'idle', nodeCount: 5, isFavorite: false,
    },
    {
      id: 'workflow-4', name: '批量下载任务', description: '批量下载文件到指定目录',
      folderId: 'folder-2', lastModified: new Date(Date.now() - 86400000),
      lastRun: new Date(Date.now() - 43200000), status: 'error', nodeCount: 8,
      isFavorite: true,
    },
    {
      id: 'workflow-5', name: '定时截图监控', description: '定时截取屏幕并比对变化',
      lastModified: new Date(Date.now() - 172800000), status: 'idle', nodeCount: 4,
      isFavorite: false,
    },
    {
      id: 'workflow-6', name: '邮件自动发送', description: '检测条件后自动发送邮件通知',
      folderId: 'folder-2', lastModified: new Date(Date.now() - 259200000),
      status: 'success', nodeCount: 6, isFavorite: false,
    },
  ]
}

function createInitialFolders(): FolderData[] {
  return [
    { id: 'folder-1', name: 'Web 自动化', isExpanded: true },
    { id: 'folder-2', name: '系统任务', isExpanded: false },
  ]
}

function createInitialNodes(): WorkflowNodeData[] {
  return [
    { id: 'node-1', type: 'find', label: '查找登录按钮', status: 'success', config: {}, x: 100, y: 80 },
    { id: 'node-2', type: 'click', label: '点击登录', status: 'success', config: {}, x: 100, y: 200 },
    { id: 'node-3', type: 'wait', label: '等待页面加载', status: 'running', config: {}, x: 100, y: 320 },
    { id: 'node-4', type: 'find', label: '查找用户名输入框', status: 'idle', config: {}, x: 100, y: 440 },
    { id: 'node-5', type: 'type', label: '输入用户名', status: 'idle', config: {}, x: 100, y: 560 },
    { id: 'node-6', type: 'condition', label: '检查登录状态', status: 'idle', config: {}, x: 350, y: 200 },
    { id: 'node-7', type: 'loop', label: '重试循环', status: 'idle', config: {}, x: 350, y: 320 },
  ]
}

function createInitialConnections(): Connection[] {
  return [
    { from: 'node-1', to: 'node-2', fromPort: 'bottom' as const, toPort: 'top' as const },
    { from: 'node-2', to: 'node-3', fromPort: 'bottom' as const, toPort: 'top' as const },
    { from: 'node-3', to: 'node-4', fromPort: 'bottom' as const, toPort: 'top' as const },
    { from: 'node-4', to: 'node-5', fromPort: 'bottom' as const, toPort: 'top' as const },
    { from: 'node-2', to: 'node-6', fromPort: 'bottom' as const, toPort: 'top' as const },
    { from: 'node-6', to: 'node-7', fromPort: 'bottom' as const, toPort: 'top' as const },
  ]
}

function createInitialImageTargets(): ImageTargetData[] {
  return [
    { id: 'target-1', name: '登录按钮', thumbnail: '', matchCount: 24, lastUsed: '2 分钟前', confidence: 0.95 },
    { id: 'target-2', name: '用户名输入框', thumbnail: '', matchCount: 18, lastUsed: '5 分钟前', confidence: 0.88 },
    { id: 'target-3', name: '密码输入框', thumbnail: '', matchCount: 12, lastUsed: '5 分钟前', confidence: 0.92 },
    { id: 'target-4', name: '提交按钮', thumbnail: '', matchCount: 8, lastUsed: '10 分钟前', confidence: 0.67 },
  ]
}

function createInitialLogs(): LogEntry[] {
  const now = Date.now()
  return [
    { id: 'log-1', timestamp: new Date(now - 5000), level: 'info', message: '工作流开始执行', details: 'Login Automation v1.2' },
    { id: 'log-2', timestamp: new Date(now - 4000), level: 'success', message: '图片匹配成功', nodeId: 'node-1', nodeName: '查找登录按钮', details: '置信度: 95%, 位置: (450, 320)' },
    { id: 'log-3', timestamp: new Date(now - 3500), level: 'info', message: '鼠标移动', nodeId: 'node-2', nodeName: '点击登录', details: '目标位置: (450, 320)' },
    { id: 'log-4', timestamp: new Date(now - 3000), level: 'success', message: '点击完成', nodeId: 'node-2', nodeName: '点击登录', details: '左键单击' },
    { id: 'log-5', timestamp: new Date(now - 2000), level: 'info', message: '等待中...', nodeId: 'node-3', nodeName: '等待页面加载', details: '剩余 1500ms' },
    { id: 'log-6', timestamp: new Date(now - 1000), level: 'warning', message: '页面加载较慢', nodeId: 'node-3', nodeName: '等待页面加载', details: '建议增加超时时间' },
  ]
}

// ============================================================
// Composable 入口
// ============================================================

export function useAutomation() {
  // ---- 工作流管理状态 ----
  const workflows = ref<WorkflowData[]>(createInitialWorkflows())
  const folders = ref<FolderData[]>(createInitialFolders())
  const currentWorkflowId = ref<string>('workflow-1')
  const isWorkflowManagerOpen = ref(false)

  // ---- 当前工作流状态 ----
  const nodes = ref<WorkflowNodeData[]>(createInitialNodes())
  const connections = ref<Connection[]>(createInitialConnections())
  const selectedNodeId = ref<string | null>('node-3')

  // ---- 图片目标状态 ----
  const imageTargets = ref<ImageTargetData[]>(createInitialImageTargets())
  const selectedTargetId = ref<string | null>(null)

  // ---- 执行状态 ----
  const isRunning = ref(true)
  const isPaused = ref(false)
  const isDebugMode = ref(true)
  const hasUnsavedChanges = ref(true)

  // ---- 日志状态 ----
  const logs = ref<LogEntry[]>(createInitialLogs())
  const selectedLogId = ref<string | null>(null)

  // ---- 屏幕调试状态 ----
  const screenshot = ref<string | null>(null)
  const matchResults = ref<MatchResult[]>([])
  const isSearching = ref(false)
  const selectedMatchId = ref<string | null>(null)

  // ---- 计算属性 ----
  const currentWorkflow = computed(() =>
    workflows.value.find((w) => w.id === currentWorkflowId.value) ?? null
  )

  const recentWorkflows = computed(() =>
    workflows.value
      .filter((w) => w.id !== currentWorkflowId.value)
      .sort((a, b) => b.lastModified.getTime() - a.lastModified.getTime())
      .slice(0, 5)
  )

  const selectedNode = computed(() =>
    nodes.value.find((n) => n.id === selectedNodeId.value) ?? null
  )

  // ---- 工具函数 ----
  function addLog(level: LogEntry['level'], message: string, details?: string, nodeId?: string, nodeName?: string) {
    logs.value = [
      ...logs.value,
      {
        id: `log-${Date.now()}`,
        timestamp: new Date(),
        level,
        message,
        details,
        nodeId,
        nodeName,
      },
    ]
  }

  function touchWorkflow() {
    workflows.value = workflows.value.map((w) =>
      w.id === currentWorkflowId.value ? { ...w, lastModified: new Date() } : w
    )
    hasUnsavedChanges.value = true
  }

  // ----------------------------------------------------------
  // 工作流管理操作
  // ----------------------------------------------------------

  function selectWorkflow(id: string) {
    currentWorkflowId.value = id
    hasUnsavedChanges.value = false
    addLog('info', `已切换到工作流: ${workflows.value.find((w) => w.id === id)?.name}`)
  }

  function createWorkflow(name: string, folderId?: string) {
    const wf: WorkflowData = {
      id: `workflow-${Date.now()}`, name, folderId, lastModified: new Date(),
      status: 'idle', nodeCount: 0, isFavorite: false,
    }
    workflows.value = [...workflows.value, wf]
    currentWorkflowId.value = wf.id
    nodes.value = []
    hasUnsavedChanges.value = false
    addLog('success', `已创建新工作流: ${name}`)
  }

  function deleteWorkflow(id: string) {
    workflows.value = workflows.value.filter((w) => w.id !== id)
    if (currentWorkflowId.value === id) {
      const remaining = workflows.value.filter((w) => w.id !== id)
      if (remaining.length > 0) currentWorkflowId.value = remaining[0].id
    }
    addLog('info', '工作流已删除')
  }

  function duplicateWorkflow(id: string) {
    const wf = workflows.value.find((w) => w.id === id)
    if (!wf) return
    const newWf: WorkflowData = { ...wf, id: `workflow-${Date.now()}`, name: `${wf.name} (副本)`, lastModified: new Date(), status: 'idle', isFavorite: false }
    workflows.value = [...workflows.value, newWf]
    addLog('success', `已复制工作流: ${wf.name}`)
  }

  function renameWorkflow(id: string, newName: string) {
    workflows.value = workflows.value.map((w) => (w.id === id ? { ...w, name: newName, lastModified: new Date() } : w))
  }

  function runWorkflow(id: string) {
    workflows.value = workflows.value.map((w) =>
      w.id === id ? { ...w, status: 'running' as const, lastRun: new Date() } : w
    )
    if (id !== currentWorkflowId.value) selectWorkflow(id)
    isRunning.value = true
    isPaused.value = false
  }

  function toggleFavorite(id: string) {
    workflows.value = workflows.value.map((w) => (w.id === id ? { ...w, isFavorite: !w.isFavorite } : w))
  }

  function createFolder(name: string) {
    folders.value = [...folders.value, { id: `folder-${Date.now()}`, name, isExpanded: true }]
  }

  function deleteFolder(id: string) {
    folders.value = folders.value.filter((f) => f.id !== id)
    workflows.value = workflows.value.map((w) => (w.folderId === id ? { ...w, folderId: undefined } : w))
  }

  function renameFolder(id: string, newName: string) {
    folders.value = folders.value.map((f) => (f.id === id ? { ...f, name: newName } : f))
  }

  function toggleFolderExpand(id: string) {
    folders.value = folders.value.map((f) => (f.id === id ? { ...f, isExpanded: !f.isExpanded } : f))
  }

  function moveToFolder(workflowId: string, folderId: string | null) {
    workflows.value = workflows.value.map((w) =>
      w.id === workflowId ? { ...w, folderId: folderId || undefined } : w
    )
  }

  function importWorkflow() {
    addLog('info', '导入功能：选择工作流文件')
  }

  function exportWorkflow(id: string) {
    const wf = workflows.value.find((w) => w.id === id)
    addLog('success', `已导出工作流: ${wf?.name}`)
  }

  // ----------------------------------------------------------
  // 节点操作
  // ----------------------------------------------------------

  function selectNode(id: string | null) {
    selectedNodeId.value = id
  }

  function updateNode(id: string, updates: Partial<WorkflowNodeData>) {
    nodes.value = nodes.value.map((node) => (node.id === id ? { ...node, ...updates } : node))
    touchWorkflow()
  }

  function addNode(type: NodeType, x: number, y: number) {
    const labels: Record<NodeType, string> = {
      find: '查找图片', mouse: '移动鼠标', click: '点击', type: '输入文本',
      wait: '等待', condition: '条件分支', loop: '循环',
    }

    const newNode: WorkflowNodeData = {
      id: `node-${Date.now()}`, type, label: labels[type],
      status: 'idle', config: {}, x, y,
    }
    nodes.value = [...nodes.value, newNode]
    selectedNodeId.value = newNode.id
    touchWorkflow()
    workflows.value = workflows.value.map((w) =>
      w.id === currentWorkflowId.value
        ? { ...w, nodeCount: w.nodeCount + 1, lastModified: new Date() } : w
    )
  }

  function deleteSelectedNode() {
    if (!selectedNodeId.value) return
    nodes.value = nodes.value.filter((n) => n.id !== selectedNodeId.value)
    selectedNodeId.value = null
    touchWorkflow()
    workflows.value = workflows.value.map((w) =>
      w.id === currentWorkflowId.value
        ? { ...w, nodeCount: Math.max(0, w.nodeCount - 1), lastModified: new Date() } : w
    )
  }

  function duplicateSelectedNode() {
    if (!selectedNodeId.value) return
    const node = nodes.value.find((n) => n.id === selectedNodeId.value)
    if (!node) return
    const newNode: WorkflowNodeData = { ...node, id: `node-${Date.now()}`, x: node.x + 30, y: node.y + 30, status: 'idle' }
    nodes.value = [...nodes.value, newNode]
    selectedNodeId.value = newNode.id
    touchWorkflow()
    workflows.value = workflows.value.map((w) =>
      w.id === currentWorkflowId.value
        ? { ...w, nodeCount: w.nodeCount + 1, lastModified: new Date() } : w
    )
  }

  // ----------------------------------------------------------
  // 图片目标操作
  // ----------------------------------------------------------

  function addImageTarget() {
    const target: ImageTargetData = {
      id: `target-${Date.now()}`, name: '新目标', thumbnail: '',
      matchCount: 0, lastUsed: '刚刚', confidence: 0,
    }
    imageTargets.value = [...imageTargets.value, target]
    selectedTargetId.value = target.id
  }

  function deleteImageTarget(id: string) {
    imageTargets.value = imageTargets.value.filter((t) => t.id !== id)
    if (selectedTargetId.value === id) selectedTargetId.value = null
  }

  function duplicateImageTarget(id: string) {
    const target = imageTargets.value.find((t) => t.id === id)
    if (!target) return
    const newTarget: ImageTargetData = { ...target, id: `target-${Date.now()}`, name: `${target.name} (副本)` }
    imageTargets.value = [...imageTargets.value, newTarget]
    selectedTargetId.value = newTarget.id
  }

  // ----------------------------------------------------------
  // 执行控制
  // ----------------------------------------------------------

  function run() {
    isRunning.value = true
    isPaused.value = false
    workflows.value = workflows.value.map((w) =>
      w.id === currentWorkflowId.value ? { ...w, status: 'running' as const, lastRun: new Date() } : w
    )
    addLog('info', '工作流继续执行')
  }

  function pause() {
    isPaused.value = true
    addLog('warning', '工作流已暂停')
  }

  function stop() {
    isRunning.value = false
    isPaused.value = false
    nodes.value = nodes.value.map((n) => ({ ...n, status: 'idle' as const }))
    workflows.value = workflows.value.map((w) =>
      w.id === currentWorkflowId.value ? { ...w, status: 'idle' as const } : w
    )
    addLog('info', '工作流已停止')
  }

  function stepForward() {
    addLog('info', '单步执行')
  }

  function toggleDebug() {
    isDebugMode.value = !isDebugMode.value
  }

  function save() {
    hasUnsavedChanges.value = false
    workflows.value = workflows.value.map((w) =>
      w.id === currentWorkflowId.value ? { ...w, lastModified: new Date() } : w
    )
    addLog('success', '工作流已保存')
  }

  function clearLogs() {
    logs.value = []
  }

  function exportLogs() {
    const text = logs.value
      .map((l) => `[${l.timestamp.toISOString()}] ${l.level}: ${l.message}`)
      .join('\n')
    const blob = new Blob([text], { type: 'text/plain' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = 'execution-logs.txt'
    a.click()
    URL.revokeObjectURL(url)
  }

  // ----------------------------------------------------------
  // 屏幕调试
  // ----------------------------------------------------------

  function capture() {
    screenshot.value = '/api/placeholder/1920/1080'
    addLog('info', '屏幕截图完成')
  }

  function search() {
    isSearching.value = true
    setTimeout(() => {
      matchResults.value = [
        { id: 'match-1', region: { x: 450, y: 320, width: 120, height: 40 }, confidence: 0.95, timestamp: new Date() },
        { id: 'match-2', region: { x: 800, y: 450, width: 120, height: 40 }, confidence: 0.72, timestamp: new Date() },
      ]
      isSearching.value = false
      addLog('success', '找到 2 个匹配结果')
    }, 1500)
  }

  return {
    // 状态
    workflows, folders, currentWorkflowId, isWorkflowManagerOpen,
    nodes, connections, selectedNodeId,
    imageTargets, selectedTargetId,
    isRunning, isPaused, isDebugMode, hasUnsavedChanges,
    logs, selectedLogId,
    screenshot, matchResults, isSearching, selectedMatchId,
    // 计算属性
    currentWorkflow, recentWorkflows, selectedNode,
    // 工作流操作
    selectWorkflow, createWorkflow, deleteWorkflow, duplicateWorkflow,
    renameWorkflow, runWorkflow, toggleFavorite,
    createFolder, deleteFolder, renameFolder, toggleFolderExpand, moveToFolder,
    importWorkflow, exportWorkflow,
    // 节点操作
    selectNode, updateNode, addNode, deleteSelectedNode, duplicateSelectedNode,
    // 图片目标操作
    addImageTarget, deleteImageTarget, duplicateImageTarget,
    // 执行控制
    run, pause, stop, stepForward, toggleDebug, save,
    clearLogs, exportLogs,
    // 屏幕调试
    capture, search,
  }
}
