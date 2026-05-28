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
      <button
        class="toolbar-btn danger"
        @click="deleteSelected"
        title="删除选中节点"
      >
        <i class="pi pi-trash"></i>
      </button>
      <div class="toolbar-divider"></div>
      <button
        class="toolbar-btn"
        :class="{ 'btn-disabled': !stepExecutable }"
        :disabled="!stepExecutable"
        @click="$emit('step-execute')"
        title="单步执行"
      >
        <i class="pi pi-step-forward"></i>
      </button>
    </div>
    <div class="canvas-wrapper" ref="graphContainer"></div>
    <TeleportContainer />
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted, nextTick } from "vue";
import { Graph, Snapline, Dnd, Selection } from "@antv/x6";
import { register, getTeleport } from "@antv/x6-vue-shape";
import VueNode from "./nodes/VueNode.vue";
import type { WorkflowNode, WorkflowEdge } from "../types";

const TeleportContainer = getTeleport();

const props = defineProps<{
  nodes: WorkflowNode[];
  edges: WorkflowEdge[];
  selectedNode: WorkflowNode | null;
  stepExecutable: boolean;
}>();

const emit = defineEmits<{
  "node-select": [node: WorkflowNode | null];
  "node-add": [node: WorkflowNode];
  "node-delete": [nodeIds: string[]];
  "edge-connect": [edge: WorkflowEdge];
  "edge-delete": [edge: { source: string; target: string }];
  "selection-change": [
    nodes: { id: string; type: string; config: Record<string, unknown> }[],
  ];
  "step-execute": [];
}>();

const containerRef = ref<HTMLElement>();
const graphContainer = ref<HTMLElement>();
const runningNodeId = ref<string | null>(null);

const NODE_WIDTH = 110;
const HEADER_HEIGHT = 40;
const NOTE_MAX_HEIGHT = 80;

let graph: Graph;
let dnd: Dnd;

// Node height helper
function computeNodeHeight(label: string): number {
  if (!label || !label.trim()) return HEADER_HEIGHT;
  const lines = label.split("\n");
  const wrappedLines = lines.reduce(
    (acc, line) => acc + Math.max(1, Math.ceil(line.length / 22)),
    0,
  );
  const contentHeight = wrappedLines * 17 + 10;
  return HEADER_HEIGHT + Math.min(contentHeight, NOTE_MAX_HEIGHT);
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
  const hasNote = node.label && node.label.trim() !== "";
  const bodyHeight = hasNote ? computeNodeHeight(node.label) : HEADER_HEIGHT;

  graph.addNode({
    id: node.id,
    shape: "vue-workflow-node",
    x: node.x,
    y: node.y,
    width: NODE_WIDTH,
    height: bodyHeight,
    data: {
      nodeType: node.type,
      nodeConfig: node.config,
      label: node.label,
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

  const newNodeIds = new Set(props.nodes.map(n => n.id));

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
      const edges = graph
        .getEdges()
        .filter(
          e => e.getSourceCellId() === source && e.getTargetCellId() === target,
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
  newVal => {
    if (!graph) return;

    // Reset previous selection
    if (previousSelectedId && previousSelectedId !== newVal?.id) {
      const oldNode = graph.getCellById(previousSelectedId);
      if (oldNode && oldNode.isNode()) {
        const oldData = oldNode.getData() ?? {};
        oldNode.setData({ ...oldData, highlighted: false });
      }
    }
    previousSelectedId = newVal?.id ?? null;

    if (newVal?.id) {
      const node = graph.getCellById(newVal.id);
      if (node && node.isNode()) {
        const data = node.getData() ?? {};

        // Update data: type/label/config + highlight
        node.setData({
          ...data,
          nodeType: newVal.type,
          nodeConfig: newVal.config,
          label: newVal.label,
          highlighted: true,
        });

        // Resize based on note content
        const hasNote = newVal.label && newVal.label.trim() !== "";
        const bodyHeight = hasNote
          ? computeNodeHeight(newVal.label)
          : HEADER_HEIGHT;
        node.resize(NODE_WIDTH, bodyHeight);
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

  if (oldId) {
    const oldNode = graph.getCellById(oldId);
    if (oldNode && oldNode.isNode()) {
      const oldData = oldNode.getData() ?? {};
      oldNode.setData({ ...oldData, running: false });
    }
  }

  if (newId) {
    const newNode = graph.getCellById(newId);
    if (newNode && newNode.isNode()) {
      const newData = newNode.getData() ?? {};
      newNode.setData({ ...newData, running: true });
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

  // Enable Dnd plugin for drag-drop from palette
  dnd = new Dnd({ target: graph });

  // Enable snapline guides for node alignment
  graph.use(
    new Snapline({
      enabled: true,
      sharp: true,
      tolerance: 10,
    }),
  );

  // Enable box selection (rubberband) — only when Alt is held
  graph.use(
    new Selection({
      enabled: true,
      multiple: true,
      rubberband: true,
      rubberNode: true,
      rubberEdge: false,
      showNodeSelectionBox: true,
      modifiers: "alt",
      selectCellOnMoved: false,
      selectNodeOnMoved: false,
      selectEdgeOnMoved: false,
    }),
  );

  // Register Vue node shape via x6-vue-shape
  register({
    shape: "vue-workflow-node",
    width: NODE_WIDTH,
    height: HEADER_HEIGHT,
    component: VueNode,
  });

  // ── Events ──────────────────────────

  graph.on("node:click", ({ node }) => {
    const data = node.getData();
    const pos = node.position();
    emit("node-select", {
      id: node.id,
      type: data.nodeType as string,
      label: (data.label as string) || "",
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
        label: (data.label as string) || "",
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

  // Track rubberband selection for step execution
  graph.on("selection:changed", () => {
    const cells = graph.getSelectedCells();
    const nodes = cells
      .filter(c => c.isNode())
      .map(c => {
        const data = c.getData();
        return {
          id: c.id,
          type: (data.nodeType as string) || "",
          config: (data.nodeConfig as Record<string, unknown>) || {},
        };
      });
    emit("selection-change", nodes);
  });

  // Handle Dnd-dropped nodes
  graph.on("node:added", ({ node }) => {
    if (isSyncing) return;
    const data = node.getData();
    const newNode: WorkflowNode = {
      id: node.id as string,
      type: (data.nodeType as string) || "unknown",
      label: (data.label as string) || "",
      x: node.getPosition().x,
      y: node.getPosition().y,
      config: (data.nodeConfig as Record<string, unknown>) || {},
    };
    graphNodeIds.add(newNode.id);
    emit("node-add", newNode);
  });

  // Initial sync
  syncNodesToGraph();
  syncEdgesToGraph();

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
  const cells = graph?.getSelectedCells() ?? [];
  const nodeIds = cells.filter(c => c.isNode()).map(c => c.id as string);

  // Also include single-click selected node if not already in selection
  if (props.selectedNode?.id && !nodeIds.includes(props.selectedNode.id)) {
    nodeIds.push(props.selectedNode.id);
  }

  if (nodeIds.length > 0) {
    emit("node-delete", nodeIds);
  }
}

// ──────────────────────────────────────
// Exposed methods
// ──────────────────────────────────────

function startDnd(blockType: string, blockLabel: string, event: MouseEvent) {
  if (!graph || !dnd) return;

  const node = graph.createNode({
    shape: "vue-workflow-node",
    width: NODE_WIDTH,
    height: HEADER_HEIGHT,
    data: { nodeType: blockType, nodeConfig: {}, label: "" },
    ports: getPorts(),
  });

  dnd.start(node, event);
}

defineExpose({
  setZoom(zoom: number) {
    graph?.zoomTo(zoom);
  },
  fitView() {
    graph?.zoomToFit({ padding: 20, maxScale: 1.5 });
  },
  startDnd,
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
  transition:
    background 0.15s,
    color 0.15s;
}

.toolbar-btn:hover {
  background: #2a2a3a;
  color: #e4e4ef;
}

.toolbar-btn.danger:hover {
  background: rgba(239, 68, 68, 0.15);
  color: #ef4444;
}

.toolbar-btn.btn-disabled {
  opacity: 0.4;
  cursor: not-allowed;
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
  0%,
  100% {
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
