<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue'
import { Graph } from '@antv/x6'
import type { WorkflowNodeData, Connection, NodeType } from '../../types'
import Button from '../ui/Button.vue'
import DynamicIcon from '../DynamicIcon.vue'

// ============================================================
// 工作流画布 — 基于 @antv/x6
// 负责节点渲染、连线展示、拖入新节点、缩放/平移
// ============================================================

const props = defineProps<{
  nodes: WorkflowNodeData[]
  connections: Connection[]
  selectedNodeId: string | null
}>()

const emit = defineEmits<{
  selectNode: [id: string | null]
  updateNode: [id: string, updates: Partial<WorkflowNodeData>]
  addNode: [type: NodeType, x: number, y: number]
}>()

const containerRef = ref<HTMLDivElement>()
const zoom = ref(1)
const isLocked = ref(false)
let graph: Graph | null = null

// ----------------------------------------------------------
// 节点宽高常量
// ----------------------------------------------------------
// Width and height constants for X6 nodes
const NODE_W = 180
const NODE_H = 80

// ----------------------------------------------------------
// 根据节点类型生成 X6 节点样式配置
// X6 的 attrs 机制可以精细控制节点内各子元素的 SVG 属性
// ----------------------------------------------------------
function nodeAttrs(type: NodeType, label: string, _status: string, isSelected: boolean) {
  const strokeColor = isSelected ? '#22c55e' : '#27272a'
  const strokeWidth = isSelected ? 2 : 1
  const bodyFill = status === 'running' ? '#052e16' : '#09090b'

  return {
    body: {
      refWidth: NODE_W,
      refHeight: NODE_H,
      fill: bodyFill,
      stroke: strokeColor,
      strokeWidth: strokeWidth,
      rx: 8,
      ry: 8,
    },
    header: {
      refWidth: NODE_W,
      refHeight: 28,
      fill: '#1a2e1a', // 默认背景色
      stroke: 'none',
      rx: 8,
      ry: 8,
    },
    label: {
      text: label,
      fill: '#e4e4e7',
      fontSize: 13,
      fontWeight: 500,
      fontFamily: 'system-ui, sans-serif',
      refX: NODE_W / 2,
      refY: 18,
      textAnchor: 'middle',
    },
    summary: {
      text: getSummary(type),
      fill: '#71717a',
      fontSize: 11,
      fontFamily: 'monospace',
      refX: NODE_W / 2,
      refY: 52,
      textAnchor: 'middle',
    },
  }
}

// ----------------------------------------------------------
// 节点类型 → 摘要文本
// ----------------------------------------------------------
function getSummary(type: NodeType): string {
  const map: Record<NodeType, string> = {
    find: '目标图片',
    mouse: '移动至 (0, 0)',
    click: '左键单击',
    type: '"文本内容"',
    wait: '等待 1000ms',
    condition: '如果找到图片...',
    loop: '重复 5 次',
  }
  return map[type]
}

// ----------------------------------------------------------
// 创建/更新单个 X6 节点
// ----------------------------------------------------------
function upsertNode(node: WorkflowNodeData) {
  if (!graph) return
  const isSelected = node.id === props.selectedNodeId

  if (graph.getCellById(node.id)) {
    // 更新已有节点
    const cell = graph.getCellById(node.id)
    if (cell.isNode()) {
      cell.setPosition({ x: node.x, y: node.y })
      cell.setAttrs(nodeAttrs(node.type, node.label, node.status, isSelected))
    }
  } else {
    // 创建新节点
    graph.addNode({
      id: node.id,
      x: node.x,
      y: node.y,
      width: NODE_W,
      height: NODE_H,
      shape: 'rect',
      attrs: nodeAttrs(node.type, node.label, node.status, isSelected),
      ports: {
        groups: {
          top: { position: { name: 'top' }, attrs: { circle: { r: 4, fill: '#27272a', stroke: '#52525b', strokeWidth: 2 } } },
          bottom: { position: { name: 'bottom' }, attrs: { circle: { r: 4, fill: '#27272a', stroke: '#52525b', strokeWidth: 2 } } },
        },
        items: [
          { id: 'top', group: 'top' },
          { id: 'bottom', group: 'bottom' },
        ],
      },
    })
  }
}

// ----------------------------------------------------------
// 创建/更新连线
// ----------------------------------------------------------
function upsertEdge(conn: Connection) {
  if (!graph) return
  const edgeId = `${conn.from}-${conn.to}`
  if (graph.getCellById(edgeId)) return // 已存在则跳过

  graph.addEdge({
    id: edgeId,
    source: { cell: conn.from, port: 'bottom' },
    target: { cell: conn.to, port: 'top' },
    attrs: {
      line: {
        stroke: '#10b981',
        strokeWidth: 2,
        targetMarker: { name: 'circle', r: 3, fill: '#10b981' },
      },
    },
    connector: { name: 'smooth' },
    zIndex: -1,
  })
}

// ----------------------------------------------------------
// 完整重绘所有节点和连线
// ----------------------------------------------------------
function renderAll() {
  if (!graph) return
  // 清除所有 cell
  graph.clearCells()

  props.connections.forEach((conn) => upsertEdge(conn))
  props.nodes.forEach((node) => upsertNode(node))
}

// ----------------------------------------------------------
// X6 Graph 初始化
// ----------------------------------------------------------
onMounted(() => {
  if (!containerRef.value) return

  graph = new Graph({
    container: containerRef.value,
    width: containerRef.value.clientWidth,
    height: containerRef.value.clientHeight,
    background: { color: 'oklch(0.1_0.005_260)' }, // 等效于 bg-[oklch(0.1_0.005_260)]
    grid: {
      size: 20,
      visible: true,
      args: { color: 'rgba(255,255,255,0.03)', thickness: 1 },
    },
    mousewheel: {
      enabled: true,
      modifiers: ['ctrl'],
      zoomAtMousePosition: true,
    },
    panning: {
      enabled: true,
      modifiers: ['alt'],
    },
  })

  // ---- 事件: 节点被点击 ----
  graph.on('node:click', ({ node }) => {
    if (typeof node.id === 'string') {
      emit('selectNode', node.id)
    }
  })

  // ---- 事件: 点击空白区域取消选中 ----
  graph.on('blank:click', () => {
    emit('selectNode', null)
  })

  // ---- 事件: 节点移动结束 ----
  graph.on('node:moved', ({ node }) => {
    if (typeof node.id === 'string') {
      const pos = node.getPosition()
      emit('updateNode', node.id, { x: pos.x, y: pos.y })
    }
  })

  // ---- 事件: 从侧边栏拖入节点类型 ----
  graph.on('node:change:position', () => {
    // 仅在拖入时触发（由 drop 处理）
  })

  // ---- 监听缩放变化 ----
  graph.on('scale', ({ sx }) => {
    zoom.value = sx
  })

  // ---- 历史记录变化（支持撤销/重做） ----
  graph.on('history:change', () => {
    // graph.canUndo() / graph.canRedo() 可用来自定义撤销按钮
  })

  // ---- 初始渲染 ----
  renderAll()

  // ---- Resize 监听 ----
  const resizeObserver = new ResizeObserver(() => {
    if (graph && containerRef.value) {
      graph.resize(containerRef.value.clientWidth, containerRef.value.clientHeight)
    }
  })
  resizeObserver.observe(containerRef.value)
})

// ---- 当外部数据变化时同步更新画布 ----
watch(() => [props.nodes, props.connections], () => {
  renderAll()
}, { deep: true })

// ---- 当选中的节点变化时更新高亮 ----
watch(() => props.selectedNodeId, () => {
  if (!graph) return
  // 更新所有节点的选中态样式
  props.nodes.forEach((node) => {
    const cell = graph!.getCellById(node.id)
    if (cell && cell.isNode()) {
      cell.setAttrs(nodeAttrs(node.type, node.label, node.status, node.id === props.selectedNodeId))
    }
  })
})

// ---- 锁定/解锁 ----
watch(isLocked, (locked) => {
  if (!graph) return
  if (locked) {
    graph.disableSelection()
    graph.disablePanning()
  } else {
    graph.enableSelection()
    graph.enablePanning()
  }
})

onUnmounted(() => {
  graph?.dispose()
  graph = null
})

// ----------------------------------------------------------
// 工具栏操作：缩放、适应画布、撤销/重做
// ----------------------------------------------------------
function zoomIn() { graph?.zoom(0.1) }
function zoomOut() { graph?.zoom(-0.1) }
function fitScreen() { graph?.zoomToFit({ padding: 40 }) }
function toggleLock() { isLocked.value = !isLocked.value }

// ----------------------------------------------------------
// 拖放处理：从侧边栏拖入节点类型，在画布上创建新节点
// ----------------------------------------------------------
function handleDrop(e: DragEvent) {
  e.preventDefault()
  if (!graph || !containerRef.value) return

  const nodeType = e.dataTransfer?.getData('nodeType') as NodeType | undefined
  if (!nodeType) return

  // 将屏幕坐标转换为画布坐标
  const rect = containerRef.value.getBoundingClientRect()
  const clientPt = graph.clientToGraph(e.clientX - rect.left, e.clientY - rect.top)
  emit('addNode', nodeType, clientPt.x - NODE_W / 2, clientPt.y - NODE_H / 2)
}

function handleDragOver(e: DragEvent) {
  e.preventDefault()
}
</script>

<template>
  <div class="relative h-full w-full overflow-hidden">
    <!-- 画布工具栏 -->
    <div class="absolute top-4 right-4 z-10 flex items-center gap-1 rounded-lg border border-border bg-card/95 p-1 backdrop-blur">
      <Button variant="ghost" size="icon" class="h-8 w-8" @click="zoomOut">
        <DynamicIcon name="zoom-out" :size="16" />
      </Button>
      <span class="w-12 text-center text-xs text-muted-foreground">
        {{ Math.round(zoom * 100) }}%
      </span>
      <Button variant="ghost" size="icon" class="h-8 w-8" @click="zoomIn">
        <DynamicIcon name="zoom-in" :size="16" />
      </Button>
      <div class="mx-1 h-4 w-px bg-border" />
      <Button variant="ghost" size="icon" class="h-8 w-8" @click="fitScreen">
        <DynamicIcon name="maximize-2" :size="16" />
      </Button>
      <Button
        variant="ghost"
        size="icon"
        class="h-8 w-8"
        :class="{ 'text-primary': isLocked }"
        @click="toggleLock"
      >
        <DynamicIcon :name="isLocked ? 'lock' : 'unlock'" :size="16" />
      </Button>
    </div>

    <!-- 画布信息 -->
    <div class="absolute bottom-4 left-4 z-10 flex items-center gap-4 text-xs text-muted-foreground">
      <span>{{ nodes.length }} 个节点</span>
      <span>{{ connections.length }} 个连接</span>
      <span>Alt+拖拽 平移 | Ctrl+滚轮 缩放</span>
    </div>

    <!-- X6 Graph 挂载容器 -->
    <div
      ref="containerRef"
      class="h-full w-full"
      @drop="handleDrop"
      @dragover="handleDragOver"
    />
  </div>
</template>
