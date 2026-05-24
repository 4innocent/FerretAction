<template>
  <div class="workflow-canvas" ref="containerRef">
    <div class="canvas-toolbar">
      <button class="toolbar-btn" @click="zoomIn" title="放大">
        <i class="pi pi-plus"></i>
      </button>
      <button class="toolbar-btn" @click="zoomOut" title="缩小">
        <i class="pi pi-minus"></i>
      </button>
      <div class="toolbar-divider"></div>
      <button class="toolbar-btn" @click="fitToContent" title="适应画布">
        <i class="pi pi-window-maximize"></i>
      </button>
      <div class="toolbar-divider"></div>
      <button class="toolbar-btn danger" @click="deleteSelected" title="删除选中节点">
        <i class="pi pi-trash"></i>
      </button>
    </div>
    <div class="canvas-wrapper" ref="graphContainer"></div>
    <div
      class="drop-zone"
      :class="{ active: isDragOver }"
    >
      <div class="drop-indicator" v-show="isDragOver">
        <i class="pi pi-plus-circle"></i>
        <span>放置以添加节点</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted, nextTick } from "vue";
import { Graph, Snapline } from "@antv/x6";
import type { WorkflowNode, WorkflowEdge } from "../types";

const props = defineProps<{
  nodes: WorkflowNode[];
  edges: WorkflowEdge[];
  selectedNode: WorkflowNode | null;
}>();

const emit = defineEmits<{
  "node-select": [node: WorkflowNode | null];
  "node-add": [node: WorkflowNode];
  "node-delete": [nodeId: string];
  "edge-connect": [edge: WorkflowEdge];
  "edge-delete": [edge: { source: string; target: string }];
}>();

const containerRef = ref<HTMLElement>();
const graphContainer = ref<HTMLElement>();
const isDragOver = ref(false);
const runningNodeId = ref<string | null>(null);

const NODE_WIDTH = 200;
const NODE_HEIGHT = 80;

let graph: Graph;

// ──────────────────────────────────────
// Node type helpers (preserved from original)
// ──────────────────────────────────────
const nodeTypeIconText: Record<string, string> = {
  start: "▶",
  end: "■",
  condition: "?",
  loop: "↻",
  break: "⏏",
  "find-image": "⌕",
  "wait-image": "◉",
  "image-gone": "⊘",
  "move-mouse": "⇅",
  click: "☛",
  "double-click": "⨍",
  "right-click": "☍",
  drag: "↔",
  scroll: "↕",
  "type-text": "✎",
  hotkey: "⚡",
  "key-press": "◎",
  wait: "⏳",
  "wait-condition": "⧖",
};

const nodeTypeColor: Record<string, string> = {
  start: "#22c55e",
  end: "#22c55e",
  condition: "#ec4899",
  loop: "#ec4899",
  break: "#ec4899",
  "find-image": "#6366f1",
  "wait-image": "#6366f1",
  "image-gone": "#6366f1",
  "move-mouse": "#f59e0b",
  click: "#f59e0b",
  "double-click": "#f59e0b",
  "right-click": "#f59e0b",
  drag: "#f59e0b",
  scroll: "#f59e0b",
  "type-text": "#8b5cf6",
  hotkey: "#8b5cf6",
  "key-press": "#8b5cf6",
  wait: "#64748b",
  "wait-condition": "#475569",
};

const nodeTypeLabel: Record<string, string> = {
  start: "开始",
  end: "结束",
  condition: "条件判断",
  loop: "循环",
  break: "跳出循环",
  "find-image": "查找图像",
  "wait-image": "等待图像",
  "image-gone": "图像消失",
  "move-mouse": "移动鼠标",
  click: "点击",
  "double-click": "双击",
  "right-click": "右键点击",
  drag: "拖拽",
  scroll: "滚动",
  "type-text": "输入文本",
  hotkey: "快捷键",
  "key-press": "按键",
  wait: "等待",
  "wait-condition": "等待条件",
};

function getNodePreview(node: WorkflowNode): string {
  if (node.type === "type-text" && node.config.text) {
    return `"${node.config.text}"`;
  }
  if (node.type === "wait" && node.config.duration) {
    return `${node.config.duration}ms`;
  }
  if (node.type === "loop" && node.config.maxIterations) {
    return `最多 ${node.config.maxIterations} 次`;
  }
  return "";
}

// ──────────────────────────────────────
// Port helpers — 4 default ports on each side of the node
// ──────────────────────────────────────
const portAttrs = {
  circle: {
    r: 3.5,
    magnet: true,
    fill: "#5F95FF",
    stroke: "#5F95FF",
    strokeWidth: 1,
    opacity: 0,
  },
};

function getPorts() {
  return {
    groups: {
      top: { position: { name: "top" }, attrs: portAttrs },
      right: { position: { name: "right" }, attrs: portAttrs },
      bottom: { position: { name: "bottom" }, attrs: portAttrs },
      left: { position: { name: "left" }, attrs: portAttrs },
    },
    items: [
      { id: "top", group: "top" },
      { id: "right", group: "right" },
      { id: "bottom", group: "bottom" },
      { id: "left", group: "left" },
    ],
  };
}

// ──────────────────────────────────────
// Edge delete tool — add/remove on hover
// ──────────────────────────────────────
function showEdgeDeleteTool(edge: any) {
  edge.addTools([
    {
      name: "button-remove",
      args: {
        distance: 0.5,
        onClick({ view, btn }: any) {
          const cell = view.cell;
          const sourceId = cell.getSourceCellId();
          const targetId = cell.getTargetCellId();
          const key = `${sourceId}->${targetId}`;
          graphEdgeIds.delete(key);
          btn.parent.remove();
          cell.remove({ ui: true, toolId: btn.cid });
          emit("edge-delete", { source: sourceId, target: targetId });
        },
      },
    },
  ]);
}

function hideEdgeDeleteTool(edge: any) {
  edge.removeTools();
}

// ──────────────────────────────────────
// Add / remove nodes in graph
// ──────────────────────────────────────
function addNodeToGraph(node: WorkflowNode) {
  const color = nodeTypeColor[node.type] || "#6366f1";
  const icon = nodeTypeIconText[node.type] || "";
  const preview = getNodePreview(node);

  graph.addNode({
    id: node.id,
    shape: "workflow-node",
    x: node.x,
    y: node.y,
    width: NODE_WIDTH,
    height: NODE_HEIGHT,
    data: {
      nodeType: node.type,
      nodeConfig: node.config,
    },
    attrs: {
      header: { fill: color },
      headerIcon: { text: icon },
      headerLabel: { text: nodeTypeLabel[node.type] || node.type },
      nodeLabel: { text: node.label },
      previewText: {
        text: preview,
        display: preview ? "block" : "none",
      },
    },
    ports: getPorts(),
  });
}

// ──────────────────────────────────────
// Sync state
// ──────────────────────────────────────
let isSyncing = false;
const graphNodeIds = new Set<string>();
const graphEdgeIds = new Set<string>();

function syncNodesToGraph() {
  if (isSyncing) return;
  isSyncing = true;

  const newNodeIds = new Set(props.nodes.map((n) => n.id));

  // Remove nodes not in props
  for (const id of graphNodeIds) {
    if (!newNodeIds.has(id)) {
      graph.removeNode(id);
      graphNodeIds.delete(id);
    }
  }

  // Add new nodes only — skip existing ones to avoid re-rendering edges
  for (const node of props.nodes) {
    if (!graphNodeIds.has(node.id)) {
      addNodeToGraph(node);
      graphNodeIds.add(node.id);
    }
  }

  isSyncing = false;
}

function syncEdgesToGraph() {
  if (isSyncing) return;
  isSyncing = true;

  const edgeMap = new Map<string, WorkflowEdge>();
  for (const e of props.edges) {
    edgeMap.set(`${e.source}->${e.target}`, e);
  }

  // Remove edges not in props
  for (const id of graphEdgeIds) {
    if (!edgeMap.has(id)) {
      const [source, target] = id.split("->");
      const edges = graph.getEdges().filter(
        (e) => e.getSourceCellId() === source && e.getTargetCellId() === target,
      );
      for (const edge of edges) {
        graph.removeEdge(edge.id);
      }
      graphEdgeIds.delete(id);
    }
  }

  // Add new edges
  for (const [key, edgeData] of edgeMap) {
    if (!graphEdgeIds.has(key)) {
      graph.addEdge({
        source: { cell: edgeData.source },
        target: { cell: edgeData.target },
        connector: { name: "smooth" },
        attrs: {
          line: {
            stroke: "#5F95FF",
            strokeWidth: 2,
            targetMarker: {
              name: "block",
              width: 10,
              height: 7,
              fill: "#5F95FF",
            },
          },
        },
        labels: edgeData.label
          ? [
              {
                attrs: {
                  text: {
                    text: edgeData.label,
                    fill: "#8888a0",
                    fontSize: 11,
                  },
                },
                position: { distance: 0.5 },
              },
            ]
          : [],
      });
      graphEdgeIds.add(key);
    }
  }

  isSyncing = false;
}

// ──────────────────────────────────────
// Watch: selectedNode external changes (selection highlight + config sync)
// ──────────────────────────────────────
let previousSelectedId: string | null = null;

watch(
  () => props.selectedNode,
  (newVal) => {
    if (!graph) return;

    // Selection highlight: reset previous, set new
    if (previousSelectedId && previousSelectedId !== newVal?.id) {
      const oldNode = graph.getCellById(previousSelectedId);
      if (oldNode && oldNode.isNode()) {
        oldNode.attr("body/stroke", "#2a2a3a");
        oldNode.attr("body/strokeWidth", 1);
      }
    }
    previousSelectedId = newVal?.id ?? null;

    if (newVal?.id) {
      const node = graph.getCellById(newVal.id);
      if (node && node.isNode()) {
        node.attr("body/stroke", "#6366f1");
        node.attr("body/strokeWidth", 2);

        // Sync label, type color, and preview from props when config panel edits
        const preview = getNodePreview(newVal);
        const color = nodeTypeColor[newVal.type] || "#6366f1";
        node.attr("nodeLabel/text", newVal.label);
        node.attr("header/fill", color);
        node.attr("headerLabel/text", nodeTypeLabel[newVal.type] || newVal.type);
        node.attr("headerIcon/text", nodeTypeIconText[newVal.type] || "");
        node.attr("previewText/text", preview);
        node.attr("previewText/display", preview ? "block" : "none");
        node.setData({ nodeType: newVal.type, nodeConfig: newVal.config });
      }
    }
  },
  { deep: true },
);

// ──────────────────────────────────────
// Watch: running node animation
// ──────────────────────────────────────
watch(runningNodeId, (newId, oldId) => {
  if (!graph) return;

  // Reset old running node
  if (oldId) {
    const oldNode = graph.getCellById(oldId);
    if (oldNode && oldNode.isNode()) {
      oldNode.attr("body/stroke", oldId === props.selectedNode?.id ? "#6366f1" : "#2a2a3a");
      oldNode.attr("body/strokeWidth", oldId === props.selectedNode?.id ? 2 : 1);
    }
  }

  // Highlight new running node with cyan border
  if (newId) {
    const newNode = graph.getCellById(newId);
    if (newNode && newNode.isNode()) {
      newNode.attr("body/stroke", "#22d3ee");
      newNode.attr("body/strokeWidth", 2);
    }
  }
});

// ──────────────────────────────────────
// Graph initialization
// ──────────────────────────────────────
onMounted(async () => {
  await nextTick();

  graph = new Graph({
    container: graphContainer.value!,
    width: containerRef.value!.clientWidth,
    height: containerRef.value!.clientHeight,
    autoResize: true,
    background: { color: "#0d0d12" },
    grid: {
      size: 20,
      visible: true,
      type: "dot",
      args: { color: "#2a2a3a", thickness: 1 },
    },
    panning: true,
    mousewheel: {
      enabled: true,
      modifiers: ["ctrl", "meta"],
      minScale: 0.3,
      maxScale: 2,
    },
    interacting: {
      nodeMovable: true,
      edgeLabelMovable: false,
    },
    connecting: {
      snap: { radius: 20 },
      allowBlank: false,
      allowLoop: false,
      highlight: true,
      connector: { name: "smooth" },
    },
  });

  // Enable snapline guides for node alignment
  graph.use(
    new Snapline({
      enabled: true,
      sharp: true,
      tolerance: 10,
    }),
  );

  // Register custom node shape
  Graph.registerNode("workflow-node", {
    width: NODE_WIDTH,
    height: NODE_HEIGHT,
    markup: [
      {
        tagName: "rect",
        selector: "body",
      },
      {
        tagName: "rect",
        selector: "header",
      },
      {
        tagName: "text",
        selector: "headerIcon",
      },
      {
        tagName: "text",
        selector: "headerLabel",
      },
      {
        tagName: "text",
        selector: "nodeLabel",
      },
      {
        tagName: "text",
        selector: "previewText",
      },
    ],
    attrs: {
      body: {
        width: NODE_WIDTH,
        height: NODE_HEIGHT,
        fill: "#16161f",
        stroke: "#2a2a3a",
        strokeWidth: 1,
        rx: 10,
        ry: 10,
      },
      header: {
        width: NODE_WIDTH,
        height: 34,
        fill: "#6366f1",
        rx: 10,
        ry: 10,
      },
      headerIcon: {
        x: 12,
        y: 22,
        fill: "#ffffff",
        fontSize: 13,
        fontWeight: 600,
        fontFamily: "monospace",
        textAnchor: "left",
      },
      headerLabel: {
        x: 32,
        y: 22,
        fill: "#ffffff",
        fontSize: 11,
        fontWeight: 600,
        fontFamily: "sans-serif",
        textAnchor: "left",
      },
      nodeLabel: {
        x: 12,
        y: 55,
        fill: "#e4e4ef",
        fontSize: 13,
        fontWeight: 500,
        fontFamily: "sans-serif",
        textAnchor: "left",
      },
      previewText: {
        x: 12,
        y: 72,
        fill: "#8888a0",
        fontSize: 11,
        fontFamily: "monospace",
        textAnchor: "left",
        display: "none",
      },
    },
  });

  // ── Events ──────────────────────────

  graph.on("node:click", ({ node }) => {
    const data = node.getData();
    const pos = node.position();
    emit("node-select", {
      id: node.id,
      type: data.nodeType as string,
      label: (node.attr("nodeLabel/text") as string) || "",
      x: pos.x,
      y: pos.y,
      config: (data.nodeConfig as Record<string, unknown>) || {},
    });
  });

  graph.on("blank:click", () => {
    emit("node-select", null);
  });

  graph.on("edge:connected", ({ edge, isNew }) => {
    if (!isNew) return;
    edge.setAttrs({
      line: {
        stroke: "#5F95FF",
        strokeWidth: 2,
        targetMarker: {
          name: "block",
          width: 10,
          height: 7,
          fill: "#5F95FF",
        },
      },
    });
    const sourceId = edge.getSourceCellId();
    const targetId = edge.getTargetCellId();
    const key = `${sourceId}->${targetId}`;
    graphEdgeIds.add(key);
    emit("edge-connect", { source: sourceId, target: targetId });
  });

  // Sync position changes back to emit for parent persistence
  graph.on("node:change:position", ({ node, current }) => {
    if (isSyncing || !current) return;
    const data = node.getData();
    if (props.selectedNode?.id === node.id) {
      emit("node-select", {
        id: node.id,
        type: data.nodeType as string,
        label: (node.attr("nodeLabel/text") as string) || "",
        x: current.x,
        y: current.y,
        config: (data.nodeConfig as Record<string, unknown>) || {},
      });
    }
  });

  // Show/hide delete tool on edge hover
  graph.on("edge:mouseenter", ({ edge }) => {
    showEdgeDeleteTool(edge);
  });
  graph.on("edge:mouseleave", ({ edge }) => {
    hideEdgeDeleteTool(edge);
  });

  // Initial sync
  syncNodesToGraph();
  syncEdgesToGraph();

  // Attach drop listeners
  attachDropListeners();

  // Watch for props changes
  watch(
    () => props.nodes,
    () => syncNodesToGraph(),
    { deep: true },
  );
  watch(
    () => props.edges,
    () => syncEdgesToGraph(),
    { deep: true },
  );
});

// ──────────────────────────────────────
// Drop handling (native DOM listeners for reliability)
// ──────────────────────────────────────
let dragCounter = 0;

function attachDropListeners() {
  const el = containerRef.value!;

  el.addEventListener("dragenter", (event: DragEvent) => {
    event.preventDefault();
    dragCounter++;
    isDragOver.value = true;
  });

  el.addEventListener("dragleave", () => {
    dragCounter--;
    if (dragCounter <= 0) {
      dragCounter = 0;
      isDragOver.value = false;
    }
  });

  el.addEventListener("dragover", (event: DragEvent) => {
    event.preventDefault();
  });

  el.addEventListener("drop", (event: DragEvent) => {
    event.preventDefault();
    dragCounter = 0;
    isDragOver.value = false;

    const data = event.dataTransfer?.getData("application/json");
    if (!data) return;

    const parsed = JSON.parse(data);
    if (parsed.type !== "action-block") return;

    const clientPoint = { x: event.clientX, y: event.clientY };
    const localPoint = graph.clientToLocal(clientPoint);

    const newNode: WorkflowNode = {
      id: `${parsed.blockType}-${Date.now()}`,
      type: parsed.blockType,
      label: parsed.label,
      x: localPoint.x - NODE_WIDTH / 2,
      y: localPoint.y - NODE_HEIGHT / 2,
      config: {},
    };

    addNodeToGraph(newNode);
    graphNodeIds.add(newNode.id);
    emit("node-add", newNode);
  });
}

// ──────────────────────────────────────
// Toolbar handlers
// ──────────────────────────────────────
function zoomIn() {
  graph?.zoom(0.1);
}

function zoomOut() {
  graph?.zoom(-0.1);
}

function fitToContent() {
  graph?.zoomToFit({ padding: 40, maxScale: 1.5 });
}

function deleteSelected() {
  if (props.selectedNode?.id) {
    emit("node-delete", props.selectedNode.id);
  }
}

// ──────────────────────────────────────
// Exposed methods
// ──────────────────────────────────────
defineExpose({
  setZoom(zoom: number) {
    graph?.zoomTo(zoom);
  },
  fitView() {
    graph?.zoomToFit({ padding: 20, maxScale: 1.5 });
  },
});

// ──────────────────────────────────────
// Cleanup
// ──────────────────────────────────────
onUnmounted(() => {
  graph?.dispose();
});
</script>

<style scoped>
.workflow-canvas {
  width: 100%;
  height: 100%;
  position: relative;
  overflow: hidden;
  background: #0d0d12;
}

.canvas-wrapper {
  width: 100%;
  height: 100%;
}

.canvas-toolbar {
  position: absolute;
  top: 8px;
  left: 8px;
  z-index: 20;
  display: flex;
  align-items: center;
  gap: 2px;
  padding: 4px;
  background: #16161f;
  border: 1px solid #2a2a3a;
  border-radius: 8px;
}

.toolbar-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: #a0a0b0;
  cursor: pointer;
  transition: background 0.15s, color 0.15s;
}

.toolbar-btn:hover {
  background: #2a2a3a;
  color: #e4e4ef;
}

.toolbar-btn.danger:hover {
  background: rgba(239, 68, 68, 0.15);
  color: #ef4444;
}

.toolbar-divider {
  width: 1px;
  height: 20px;
  background: #2a2a3a;
  margin: 0 2px;
}

.drop-zone {
  position: absolute;
  inset: 0;
  z-index: 10;
  pointer-events: none;
}

.drop-zone.active {
  background: rgba(99, 102, 241, 0.1);
  border: 2px dashed #6366f1;
}

.drop-indicator {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  color: #6366f1;
}

.drop-indicator i {
  font-size: 2rem;
}
</style>

<style>
/* X6 overrides - must be global (not scoped) to affect X6 SVG elements */

.x6-graph {
  background: #0d0d12;
}

/* Ports hidden by default, shown on node hover */
.x6-port-body {
  opacity: 0;
  transition: opacity 0.15s;
}

.x6-node:hover .x6-port-body {
  opacity: 1;
}

/* Running state pulse via CSS animation */
.x6-node.running-pulse .x6-node-body rect:nth-child(1) {
  animation: x6-pulse 1.5s infinite;
}

@keyframes x6-pulse {
  0%, 100% {
    stroke: #22d3ee;
    stroke-width: 1;
    filter: none;
  }
  50% {
    stroke: #22d3ee;
    stroke-width: 3;
    filter: drop-shadow(0 0 6px rgba(34, 211, 238, 0.4));
  }
}

/* Selection rubber band */
.x6-widget-selection-box {
  border: 2px dashed #6366f1;
  background: rgba(99, 102, 241, 0.05);
}

/* Tool buttons visibility on node hover */
.x6-node .x6-node-tool {
  display: none;
}

.x6-node:hover .x6-node-tool {
  display: block;
}

/* Smooth edge style */
.x6-edge path.x6-edge-line {
  stroke-linecap: round;
  stroke-linejoin: round;
}
</style>
