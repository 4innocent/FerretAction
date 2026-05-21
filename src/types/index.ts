// ============================================================
// 自动化工作流 - 类型定义
// ============================================================

/** 节点类型 */
export type NodeType = 'find' | 'mouse' | 'click' | 'type' | 'wait' | 'condition' | 'loop'

/** 节点运行状态 */
export type NodeStatus = 'idle' | 'running' | 'success' | 'error'

/** 工作流运行状态 */
export type WorkflowStatus = 'idle' | 'running' | 'success' | 'error'

/** 日志级别 */
export type LogLevel = 'info' | 'success' | 'warning' | 'error'

// ----------------------------------------------------------
// 工作流节点
// ----------------------------------------------------------

export interface WorkflowNodeData {
  id: string
  type: NodeType
  label: string
  status: NodeStatus
  config: Record<string, unknown>
  x: number
  y: number
}

// ----------------------------------------------------------
// 连线
// ----------------------------------------------------------

export interface Connection {
  from: string
  to: string
  fromPort: 'bottom'
  toPort: 'top'
}

// ----------------------------------------------------------
// 工作流
// ----------------------------------------------------------

export interface WorkflowData {
  id: string
  name: string
  description?: string
  folderId?: string
  lastModified: Date
  lastRun?: Date
  status: WorkflowStatus
  nodeCount: number
  isFavorite: boolean
  tags?: string[]
}

// ----------------------------------------------------------
// 文件夹
// ----------------------------------------------------------

export interface FolderData {
  id: string
  name: string
  isExpanded: boolean
  color?: string
}

// ----------------------------------------------------------
// 图片目标
// ----------------------------------------------------------

export interface ImageTargetData {
  id: string
  name: string
  thumbnail: string
  matchCount: number
  lastUsed: string
  confidence: number
}

// ----------------------------------------------------------
// 执行日志
// ----------------------------------------------------------

export interface LogEntry {
  id: string
  timestamp: Date
  level: LogLevel
  message: string
  nodeId?: string
  nodeName?: string
  details?: string
  screenshot?: string
}

// ----------------------------------------------------------
// 屏幕区域
// ----------------------------------------------------------

export interface ScreenRegion {
  x: number
  y: number
  width: number
  height: number
}

// ----------------------------------------------------------
// 屏幕匹配结果
// ----------------------------------------------------------

export interface MatchResult {
  id: string
  region: ScreenRegion
  confidence: number
  timestamp: Date
}

// ----------------------------------------------------------
// 节点配置元数据（图标、颜色映射）
// ----------------------------------------------------------

export interface NodeTypeConfig {
  icon: string
  label: string
  description: string
  color: string
  bgColor: string
}

/** 节点类型 → 配置映射表 */
export const NODE_TYPE_CONFIG: Record<NodeType, NodeTypeConfig> = {
  find: {
    icon: 'search',
    label: '查找图片',
    description: '在屏幕上搜索图片',
    color: 'text-[oklch(0.72_0.18_165)]',
    bgColor: 'bg-[oklch(0.72_0.18_165/0.15)]',
  },
  mouse: {
    icon: 'mouse-pointer',
    label: '移动鼠标',
    description: '移动鼠标到指定位置',
    color: 'text-[oklch(0.65_0.15_250)]',
    bgColor: 'bg-[oklch(0.65_0.15_250/0.15)]',
  },
  click: {
    icon: 'mouse-pointer-click',
    label: '点击',
    description: '执行鼠标点击',
    color: 'text-[oklch(0.7_0.2_30)]',
    bgColor: 'bg-[oklch(0.7_0.2_30/0.15)]',
  },
  type: {
    icon: 'keyboard',
    label: '输入文本',
    description: '模拟键盘输入',
    color: 'text-[oklch(0.65_0.18_300)]',
    bgColor: 'bg-[oklch(0.65_0.18_300/0.15)]',
  },
  wait: {
    icon: 'clock',
    label: '等待',
    description: '暂停执行',
    color: 'text-[oklch(0.6_0.12_60)]',
    bgColor: 'bg-[oklch(0.6_0.12_60/0.15)]',
  },
  condition: {
    icon: 'git-branch',
    label: '条件分支',
    description: '根据条件执行不同操作',
    color: 'text-[oklch(0.68_0.16_200)]',
    bgColor: 'bg-[oklch(0.68_0.16_200/0.15)]',
  },
  loop: {
    icon: 'repeat',
    label: '循环',
    description: '重复执行操作',
    color: 'text-[oklch(0.65_0.15_280)]',
    bgColor: 'bg-[oklch(0.65_0.15_280/0.15)]',
  },
}

/** 侧边栏可拖拽的节点块列表 */
export const NODE_BLOCKS = Object.entries(NODE_TYPE_CONFIG).map(([type, config]) => ({
  type: type as NodeType,
  label: config.label,
  description: config.description,
}))
