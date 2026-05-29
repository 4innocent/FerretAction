<template>
  <div class="app-container">
    <TopBar
      :workflow-name="workflowName"
      :connection-status="connectionStatus"
      :connection-status-text="connectionStatusText"
      @toggle-settings="showSettings = true"
      @save="onSaveClick"
    />

    <div class="main-content">
      <LeftSidebar
        :sidebar-width="leftSidebarWidth"
        :image-targets="imageTargets"
        :active-workflow-id="currentWorkflowId"
        @workflow-select="onWorkflowSelect"
        @workflow-run="onWorkflowRun"
        @workflow-create="onWorkflowCreate"
        @workflow-delete="onWorkflowDelete"
        @image-add="addImageTarget"
        @image-delete="deleteImageTarget"
        @target-rename="renameTarget"
        @drag-start="(block, event) => onDragStart(block, event)"
      />

      <div class="resizer resizer-left" @mousedown="startResizeLeft"></div>

      <main class="canvas-area">
        <WorkflowCanvas
          ref="canvasRef"
          :nodes="workflowNodes"
          :edges="workflowEdges"
          :selected-node="selectedNode"
          :step-executable="canStepExecute"
          @node-select="onNodeSelect"
          @node-add="onNodeAdd"
          @node-delete="onNodeDelete"
          @edge-connect="onEdgeConnect"
          @edge-delete="onEdgeDelete"
          @selection-change="onSelectionChange"
          @step-execute="onStepExecute"
        />

        <FloatingNodePanel
          :selected-node="selectedNode"
          :image-targets="imageTargets"
          :last-capture="lastCapture"
          @update-node-config="updateNodeConfig"
          @close="selectedNode = null"
        />
      </main>
    </div>

    <BottomPanel :execution-logs="executionLogs" @clear="executionLogs = []" />

    <Dialog
      v-model:visible="showUnsavedDialog"
      header="未保存的更改"
      :modal="true"
      :closable="false"
      :style="{ width: '400px' }"
    >
      <div class="unsaved-content">
        <i class="pi pi-exclamation-triangle"></i>
        <p>当前工作流有未保存的更改，是否保存后再切换？</p>
      </div>
      <template #footer>
        <Button
          label="不保存"
          text
          severity="danger"
          @click="handleUnsavedDiscard"
        />
        <Button label="取消" text @click="handleUnsavedCancel" />
        <Button label="保存" @click="handleUnsavedSave" />
      </template>
    </Dialog>

    <Dialog
      v-model:visible="showSettings"
      header="设置"
      :style="{ width: '560px' }"
      modal
      :contentStyle="{ padding: 0, overflow: 'hidden' }"
    >
      <SettingsPanel
        :capture-shortcut="captureShortcut"
        :quick-execute-shortcut="quickExecuteShortcut"
        :last-capture="lastCapture"
        :theme="theme"
        :mouse-duration="mouseDuration"
        :step-delay="stepDelay"
        :stop-strategy="stopStrategy"
        :minimize-on-execute="minimizeOnExecute"
        @update:capture-shortcut="onCaptureShortcutChange"
        @update:quick-execute-shortcut="onQuickExecuteShortcutChange"
        @update:theme="onThemeChange"
        @update:mouse-duration="onMouseDurationChange"
        @update:step-delay="onStepDelayChange"
        @update:stop-strategy="stopStrategy = $event; saveSetting('stopStrategy', $event)"
        @update:minimize-on-execute="minimizeOnExecute = $event; saveSetting('minimizeOnExecute', String($event))"
      />
    </Dialog>

    <Toast />
  </div>
</template>

<script setup lang="ts">
import {
  ref,
  computed,
  provide,
  watch,
  onMounted,
  onUnmounted,
  nextTick,
} from "vue";
import Dialog from "primevue/dialog";
import Toast from "primevue/toast";
import Button from "primevue/button";
import { useToast } from "primevue/usetoast";
import { useDebounceFn } from "@vueuse/core";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { register, unregister } from "@tauri-apps/plugin-global-shortcut";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";

import TopBar from "./pages/TopBar.vue";
import LeftSidebar from "./pages/LeftSidebar.vue";
import WorkflowCanvas from "./components/WorkflowCanvas.vue";
import FloatingNodePanel from "./pages/FloatingNodePanel.vue";
import BottomPanel from "./pages/BottomPanel.vue";
import SettingsPanel from "./components/SettingsPanel.vue";
import type { WorkflowNode, ImageTarget, Workflow, ExecutionLog } from "./types";
import { getWorkflow, saveWorkflowContent, initDb, loadSettings, saveSetting } from "./db";

const toast = useToast();

// Shared state
const workflowName = ref("");
const currentWorkflowId = ref<string | null>(null);
const isRunning = ref(false);
const zoomLevel = ref(1);
const selectedNode = ref<WorkflowNode | null>(null);
const showSettings = ref(false);

// Execution logs
const executionLogs = ref<ExecutionLog[]>([]);
function addLog(level: ExecutionLog["level"], message: string, nodeId?: string) {
  executionLogs.value.push({
    id: crypto.randomUUID(),
    timestamp: new Date(),
    level,
    message,
    nodeId: nodeId ?? null,
  });
}

// ── Mouse duration ───────────────────────────────────────────

const mouseDuration = ref(Number(localStorage.getItem("ferret-mouse-duration") || 100));
const stepDelay = ref(150);
const stopStrategy = ref("none");
const minimizeOnExecute = ref(true);

function onStepDelayChange(ms: number) {
  stepDelay.value = ms;
  saveSetting("stepDelay", String(ms));
}

function onMouseDurationChange(ms: number) {
  mouseDuration.value = ms;
  localStorage.setItem("ferret-mouse-duration", String(ms));
  saveSetting("mouseDuration", String(ms));
}

// ── Theme ────────────────────────────────────────────────────

const theme = ref<string>(localStorage.getItem("ferret-theme") || "dark");

function applyTheme(t: string) {
  const root = document.documentElement;
  const isDark = t === "system"
    ? window.matchMedia("(prefers-color-scheme: dark)").matches
    : t === "dark";
  root.classList.toggle("dark", isDark);
}

function onThemeChange(t: string) {
  theme.value = t;
  localStorage.setItem("ferret-theme", t);
  applyTheme(t);
  saveSetting("theme", t);
}

// Watch system preference when theme is "system"
const systemDarkQuery = window.matchMedia("(prefers-color-scheme: dark)");
systemDarkQuery.addEventListener("change", () => {
  if (theme.value === "system") applyTheme("system");
});

// Apply on startup
applyTheme(theme.value);

// ── Global shortcut ──────────────────────────────────────────
const captureShortcut = ref("Ctrl+Shift+K");
const quickExecuteShortcut = ref("Ctrl+R");
const lastCapture = ref("");
const shortcutReady = ref(false);

function toRegShortcut(sc: string) {
  return sc.replace(/^Ctrl/, "CommandOrControl");
}

async function registerShortcut(sc: string) {
  if (!shortcutReady.value) return;
  try {
    try {
      await unregister(toRegShortcut(captureShortcut.value));
    } catch (_) {}
    await register(toRegShortcut(sc), async event => {
      if (event.state === "Pressed") {
        try {
          const coords = await invoke<[number, number]>("get_mouse_location");
          const text = `(${coords[0]}, ${coords[1]})`;
          await writeText(text);
          lastCapture.value = text;
        } catch (e) {
          console.error("捕获坐标失败:", e);
        }
      }
    });
    captureShortcut.value = sc;
  } catch (e) {
    console.error("注册全局快捷键失败:", e);
  }
}

function onCaptureShortcutChange(newShortcut: string) {
  registerShortcut(newShortcut);
  saveSetting("captureShortcut", newShortcut);
}

async function registerQuickExecuteShortcut(sc: string) {
  if (!shortcutReady.value) return;
  try {
    try { await unregister(toRegShortcut(quickExecuteShortcut.value)); } catch (_) {}
    await register(toRegShortcut(sc), async event => {
      if (event.state === "Pressed" && currentWorkflowId.value) {
        try {
          const wf = await getWorkflow(currentWorkflowId.value);
          await onWorkflowRun(wf.workflow);
        } catch (e) {
          console.error("快捷执行失败:", e);
        }
      }
    });
    quickExecuteShortcut.value = sc;
  } catch (e) {
    console.error("注册快捷执行快捷键失败:", e);
  }
}

function onQuickExecuteShortcutChange(newShortcut: string) {
  registerQuickExecuteShortcut(newShortcut);
  saveSetting("quickExecuteShortcut", newShortcut);
}

onMounted(async () => {
  // Ensure DB tables exist, then load persisted settings
  await initDb();
  try {
    const saved = await loadSettings();
    if (saved.theme) { theme.value = saved.theme; applyTheme(saved.theme); }
    if (saved.mouseDuration) mouseDuration.value = Number(saved.mouseDuration);
    if (saved.stepDelay) stepDelay.value = Number(saved.stepDelay);
    if (saved.stopStrategy) stopStrategy.value = saved.stopStrategy;
    if (saved.minimizeOnExecute === "false") minimizeOnExecute.value = false;
    if (saved.captureShortcut) { captureShortcut.value = saved.captureShortcut; }
    if (saved.quickExecuteShortcut) { quickExecuteShortcut.value = saved.quickExecuteShortcut; }
  } catch (e) {
    console.error("Failed to load settings:", e);
  }

  // Defer shortcut registration to avoid blocking startup
  setTimeout(() => {
    shortcutReady.value = true;
    registerShortcut(captureShortcut.value);
    registerQuickExecuteShortcut(quickExecuteShortcut.value);
  }, 2000);
});

onUnmounted(() => {
  shortcutReady.value = false;
  try {
    unregister(toRegShortcut(captureShortcut.value));
    unregister(toRegShortcut(quickExecuteShortcut.value));
  } catch (_) {}
});

// Sidebar
const leftSidebarWidth = ref(280);
const isResizing = ref(false);
const canvasRef = ref();

// Connection Status
const connectionStatus = computed(() =>
  isRunning.value ? "connected" : "idle",
);
const connectionStatusText = computed(() =>
  isRunning.value ? "运行中" : "就绪",
);

// Image Targets (shared with FloatingNodePanel)
const imageTargets = ref<ImageTarget[]>([]);

// Workflow canvas
const workflowNodes = ref<WorkflowNode[]>([]);
const workflowEdges = ref<{ source: string; target: string; label?: string }[]>(
  [],
);
const isLoading = ref(false);
const isDirty = ref(false);
const showUnsavedDialog = ref(false);
const pendingWorkflow = ref<Workflow | null>(null);
// Snapshot of original state when workflow was first loaded / last saved
const originalNodes = ref<WorkflowNode[]>([]);
const originalEdges = ref<{ source: string; target: string; label?: string }[]>(
  [],
);

// ── Step execution ───────────────────────────────────────────

const ACTIONABLE_TYPES = new Set([
  "move-mouse",
  "click",
  "double-click",
  "scroll",
  "type-text",
  "hotkey",
  "key-press",
  "wait",
  "wait-condition",
]);

const selectedNodes = ref<
  { id: string; type: string; config: Record<string, unknown> }[]
>([]);
const canStepExecute = computed(
  () =>
    selectedNodes.value.length > 0 &&
    selectedNodes.value.some(n => ACTIONABLE_TYPES.has(n.type)),
);

function onSelectionChange(
  nodes: { id: string; type: string; config: Record<string, unknown> }[],
) {
  selectedNodes.value = nodes;
}

async function onStepExecute() {
  if (!canStepExecute.value) return;

  // Only use selected nodes — do NOT traverse the whole graph
  const selectedIds = new Set(selectedNodes.value.map(n => n.id));

  // Split selected nodes into actionable and loop
  const actionable = selectedNodes.value.filter(n => ACTIONABLE_TYPES.has(n.type));
  const loopNode = selectedNodes.value.find(n => n.type === "loop");

  if (actionable.length === 0) return;
  const iterations = loopNode ? (Number(loopNode.config.maxIterations) || 3) : 1;

  // Build adjacency only among selected nodes (edges between them)
  const nextMap = new Map<string, string[]>();
  const prevMap = new Map<string, string[]>();
  for (const e of workflowEdges.value) {
    if (selectedIds.has(e.source) && selectedIds.has(e.target)) {
      if (!nextMap.has(e.source)) nextMap.set(e.source, []);
      nextMap.get(e.source)!.push(e.target);
      if (!prevMap.has(e.target)) prevMap.set(e.target, []);
      prevMap.get(e.target)!.push(e.source);
    }
  }

  const actionableIds = new Set(actionable.map(n => n.id));

  // Find start node: actionable, no incoming from another actionable in selection
  let startId = actionable[0]?.id;
  for (const node of actionable) {
    const prevs = prevMap.get(node.id) || [];
    if (!prevs.some(p => actionableIds.has(p))) {
      startId = node.id;
      break;
    }
  }

  // BFS from start node, only through selected edges
  const ordered: typeof actionable = [];
  const visited = new Set<string>();
  const queue = [startId];
  while (queue.length > 0) {
    const id = queue.shift()!;
    if (visited.has(id)) continue;
    visited.add(id);
    const node = actionable.find(n => n.id === id);
    if (node) ordered.push(node);
    for (const next of nextMap.get(id) || []) {
      if (!visited.has(next)) queue.push(next);
    }
  }
  // Any not reached by BFS
  for (const node of actionable) {
    if (!visited.has(node.id)) ordered.push(node);
  }

  toast.add({
    severity: "info",
    summary: `执行 ${ordered.length} 个节点` + (iterations > 1 ? ` × ${iterations} 轮` : ""),
    life: 2000,
  });

  addLog("info", `开始单步执行: ${ordered.length} 个节点` + (iterations > 1 ? ` × ${iterations} 轮` : ""));
  try {
    if (minimizeOnExecute.value) {
      await getCurrentWindow().minimize();
      await new Promise(r => setTimeout(r, 300));
    }

    for (let i = 0; i < iterations; i++) {
      await invoke("execute_step", { nodes: ordered, mouseDuration: mouseDuration.value, stepDelay: stepDelay.value, stopStrategy: stopStrategy.value });
      for (const n of ordered) {
        addLog("success", `执行: ${n.type}`, n.id);
      }
      if (i < iterations - 1) {
        await new Promise(r => setTimeout(r, 200));
      }
    }
    addLog("success", `执行完成`);
  } catch (e: any) {
    addLog("error", `执行失败: ${String(e)}`);
    console.error("Step execution failed:", e);
    toast.add({
      severity: "error",
      summary: "执行失败",
      detail: String(e),
      life: 4000,
    });
  }
}

// Execution
const toggleExecution = () => {
  isRunning.value = !isRunning.value;
  if (isRunning.value) {
    toast.add({
      severity: "info",
      summary: "开始执行",
      detail: "工作流已启动",
      life: 3000,
    });
  } else {
    toast.add({
      severity: "warn",
      summary: "执行停止",
      detail: "工作流已停止",
      life: 3000,
    });
  }
};

// Image target mutations
const addImageTarget = (data: {
  name: string;
  thumbnail: string;
  folderId: string;
}) => {
  imageTargets.value.push({
    id: String(Date.now()),
    name: data.name,
    thumbnail: data.thumbnail,
    folderId: data.folderId,
    matchThreshold: 0.85,
    lastMatch: null,
  });
};

const deleteImageTarget = (id: string) => {
  imageTargets.value = imageTargets.value.filter(t => t.id !== id);
};

const renameTarget = (id: string, name: string) => {
  const target = imageTargets.value.find(t => t.id === id);
  if (target) target.name = name;
};

// ── Workflow switching with dirty check ──────────────────────

const onSaveClick = async () => {
  if (!currentWorkflowId.value) return;
  await doSave();
  addLog("success", "工作流已保存");
  toast.add({ severity: "success", summary: "已保存", life: 1500 });
};

function snapshotState() {
  originalNodes.value = JSON.parse(JSON.stringify(workflowNodes.value));
  originalEdges.value = JSON.parse(JSON.stringify(workflowEdges.value));
}

async function loadWorkflowToCanvas(workflow: Workflow) {
  isLoading.value = true;
  currentWorkflowId.value = workflow.id;
  workflowName.value = workflow.name;
  try {
    const data = await getWorkflow(workflow.id);
    workflowNodes.value = data.nodes;
    workflowEdges.value = data.edges;
    isDirty.value = false;
    snapshotState();
    addLog("info", `加载工作流: ${workflow.name} (${data.nodes.length} 节点, ${data.edges.length} 连线)`);
  } catch (e) {
    addLog("error", `加载工作流失败: ${workflow.name}`);
    console.error("Failed to load workflow:", e);
  } finally {
    await nextTick();
    isLoading.value = false;
  }
}

async function doSave() {
  if (!currentWorkflowId.value) return;
  await saveWorkflowContent(
    currentWorkflowId.value,
    workflowNodes.value,
    workflowEdges.value,
  );
  isDirty.value = false;
  snapshotState();
}

const onWorkflowSelect = async (workflow: Workflow) => {
  if (workflow.id === currentWorkflowId.value) return;
  if (isDirty.value) {
    pendingWorkflow.value = workflow;
    showUnsavedDialog.value = true;
    return;
  }
  await loadWorkflowToCanvas(workflow);
};

async function handleUnsavedSave() {
  await doSave();
  showUnsavedDialog.value = false;
  if (pendingWorkflow.value) {
    await loadWorkflowToCanvas(pendingWorkflow.value);
    pendingWorkflow.value = null;
  }
  toast.add({ severity: "success", summary: "已保存", life: 1500 });
}

async function handleUnsavedDiscard() {
  // Restore original state and persist it back to DB
  workflowNodes.value = JSON.parse(JSON.stringify(originalNodes.value));
  workflowEdges.value = JSON.parse(JSON.stringify(originalEdges.value));
  if (currentWorkflowId.value) {
    await saveWorkflowContent(
      currentWorkflowId.value,
      workflowNodes.value,
      workflowEdges.value,
    );
  }
  isDirty.value = false;
  showUnsavedDialog.value = false;
  if (pendingWorkflow.value) {
    await loadWorkflowToCanvas(pendingWorkflow.value);
    pendingWorkflow.value = null;
  }
}

function handleUnsavedCancel() {
  pendingWorkflow.value = null;
  showUnsavedDialog.value = false;
}

async function onWorkflowRun(workflow: Workflow) {
  // Ensure the workflow content is loaded
  if (workflow.id !== currentWorkflowId.value) {
    await loadWorkflowToCanvas(workflow);
  }

  // Check for start node
  const startNode = workflowNodes.value.find(n => n.type === "start");
  if (!startNode) {
    toast.add({
      severity: "warn",
      summary: "无法执行",
      detail: "工作流缺少开始节点",
      life: 4000,
    });
    return;
  }

  // Build adjacency maps from all edges
  const nextMap = new Map<string, string[]>();
  const prevMap = new Map<string, string[]>();
  for (const e of workflowEdges.value) {
    if (!nextMap.has(e.source)) nextMap.set(e.source, []);
    nextMap.get(e.source)!.push(e.target);
    if (!prevMap.has(e.target)) prevMap.set(e.target, []);
    prevMap.get(e.target)!.push(e.source);
  }

  const nodeDataMap = new Map(workflowNodes.value.map(n => [n.id, n]));

  // BFS from start node to collect all actionable + loop nodes in graph order
  const collected: { id: string; type: string; config: Record<string, unknown> }[] = [];
  let loopNode: { id: string; type: string; config: Record<string, unknown> } | undefined;
  const visited = new Set<string>();
  const queue = [startNode.id];
  while (queue.length > 0) {
    const id = queue.shift()!;
    if (visited.has(id)) continue;
    visited.add(id);
    const data = nodeDataMap.get(id);
    if (!data) continue;
    if (ACTIONABLE_TYPES.has(data.type)) {
      collected.push({ id, type: data.type, config: data.config });
    } else if (data.type === "loop") {
      loopNode = { id: "loop", type: "loop", config: data.config };
    }
    for (const next of nextMap.get(id) || []) {
      if (!visited.has(next)) queue.push(next);
    }
  }

  if (collected.length === 0) {
    toast.add({
      severity: "warn",
      summary: "无法执行",
      detail: "工作流无可执行节点",
      life: 4000,
    });
    return;
  }

  // Order by graph topology
  const collectedIds = new Set(collected.map(n => n.id));
  let startId = collected[0]?.id;
  for (const node of collected) {
    const prevs = prevMap.get(node.id) || [];
    if (!prevs.some(p => collectedIds.has(p))) { startId = node.id; break; }
  }

  const ordered: typeof collected = [];
  const orderedVisited = new Set<string>();
  const orderQueue = [startId];
  while (orderQueue.length > 0) {
    const id = orderQueue.shift()!;
    if (orderedVisited.has(id)) continue;
    orderedVisited.add(id);
    const node = collected.find(n => n.id === id);
    if (node) ordered.push(node);
    for (const next of nextMap.get(id) || []) {
      if (!orderedVisited.has(next)) orderQueue.push(next);
    }
  }
  for (const node of collected) {
    if (!orderedVisited.has(node.id)) ordered.push(node);
  }

  const iterations = loopNode ? (Number(loopNode.config.maxIterations) || 3) : 1;

  workflowName.value = workflow.name;
  isRunning.value = true;
  addLog("info", `开始执行工作流: ${workflow.name} (${ordered.length} 节点` + (iterations > 1 ? ` × ${iterations} 轮)` : ")"));
  toast.add({
    severity: "success",
    summary: "开始执行",
    detail: `正在运行: ${workflow.name}`,
    life: 3000,
  });

  try {
    if (minimizeOnExecute.value) {
      await getCurrentWindow().minimize();
      await new Promise(r => setTimeout(r, 300));
    }

    for (let i = 0; i < iterations; i++) {
      await invoke("execute_step", {
        nodes: ordered,
        mouseDuration: mouseDuration.value,
        stepDelay: stepDelay.value,
        stopStrategy: stopStrategy.value,
      });
      for (const n of ordered) {
        addLog("success", `执行: ${n.type}`, n.id);
      }
      if (i < iterations - 1) {
        await new Promise(r => setTimeout(r, 200));
      }
    }
    addLog("success", `工作流执行完成`);
  } catch (e: any) {
    addLog("error", `执行失败: ${String(e)}`);
    toast.add({
      severity: "error",
      summary: "执行失败",
      detail: String(e),
      life: 4000,
    });
  } finally {
    isRunning.value = false;
  }
}

const onWorkflowCreate = async (data: {
  name: string;
  description?: string;
  folderId?: string;
}) => {
  workflowName.value = data.name;
  const startNode: WorkflowNode = {
    id: "start",
    type: "start",
    label: "开始",
    x: 100,
    y: 200,
    config: {},
  };
  workflowNodes.value = [startNode];
  workflowEdges.value = [];
  isDirty.value = true;
  snapshotState();
};

// ── Auto-save ────────────────────────────────────────────────

const debouncedSave = useDebounceFn(async () => {
  if (!currentWorkflowId.value) return;
  try {
    await saveWorkflowContent(
      currentWorkflowId.value,
      workflowNodes.value,
      workflowEdges.value,
    );
  } catch (e) {
    console.error("Auto-save failed:", e);
  }
}, 500);

watch(
  [workflowNodes, workflowEdges],
  () => {
    if (isLoading.value) return;
    if (currentWorkflowId.value) isDirty.value = true;
    debouncedSave();
  },
  { deep: true },
);

// ── Close guard ──────────────────────────────────────────────

function onBeforeUnload(e: BeforeUnloadEvent) {
  if (isDirty.value) {
    e.preventDefault();
    e.returnValue = "";
  }
}

onMounted(() => window.addEventListener("beforeunload", onBeforeUnload));
onUnmounted(() => window.removeEventListener("beforeunload", onBeforeUnload));

// ── Workflow deletion ────────────────────────────────────────

const onWorkflowDelete = (workflowId: string) => {
  if (currentWorkflowId.value === workflowId) {
    currentWorkflowId.value = null;
    workflowName.value = "";
    workflowNodes.value = [];
    workflowEdges.value = [];
    isDirty.value = false;
    selectedNode.value = null;
  }
};

const onDragStart = (
  block: { type: string; label: string },
  event: MouseEvent,
) => {
  canvasRef.value?.startDnd(block.type, block.label, event);
};

// Canvas events
const onNodeSelect = (node: WorkflowNode | null) => {
  selectedNode.value = node;
};

const onNodeAdd = (node: WorkflowNode) => {
  workflowNodes.value.push(node);
  const typeLabel = node.type === "start" ? "开始" : node.type;
  addLog("success", `添加节点: ${typeLabel}`, node.id);
};

const onNodeDelete = (nodeIds: string[]) => {
  const idSet = new Set(nodeIds);
  workflowNodes.value = workflowNodes.value.filter(n => !idSet.has(n.id));
  workflowEdges.value = workflowEdges.value.filter(
    e => !idSet.has(e.source) && !idSet.has(e.target),
  );
  if (selectedNode.value?.id && idSet.has(selectedNode.value.id)) selectedNode.value = null;
  addLog("warning", `删除 ${nodeIds.length} 个节点`, nodeIds[0]);
};

const onEdgeConnect = (edge: { source: string; target: string }) => {
  workflowEdges.value.push(edge);
  addLog("success", `连线: ${edge.source} → ${edge.target}`, edge.source);
};

const onEdgeDelete = (edge: { source: string; target: string }) => {
  workflowEdges.value = workflowEdges.value.filter(
    e => !(e.source === edge.source && e.target === edge.target),
  );
  addLog("warning", `断开连线: ${edge.source} → ${edge.target}`);
};

const updateNodeConfig = (nodeId: string, config: Record<string, unknown>) => {
  const node = workflowNodes.value.find(n => n.id === nodeId);
  if (node) {
    if ('label' in config) {
      node.label = config.label as string;
      delete config.label;
    }
    node.config = { ...node.config, ...config };
    if (selectedNode.value?.id === nodeId) {
      selectedNode.value = { ...selectedNode.value, config: node.config, label: node.label };
    }
  }
};

// Sidebar resize
const startResizeLeft = (e: MouseEvent) => {
  isResizing.value = true;
  document.addEventListener("mousemove", handleResize);
  document.addEventListener("mouseup", stopResize);
  document.body.style.cursor = "col-resize";
  document.body.style.userSelect = "none";
};

const handleResize = (e: MouseEvent) => {
  if (!isResizing.value) return;
  leftSidebarWidth.value = Math.max(200, Math.min(500, e.clientX));
};

const stopResize = () => {
  isResizing.value = false;
  document.removeEventListener("mousemove", handleResize);
  document.removeEventListener("mouseup", stopResize);
  document.body.style.cursor = "";
  document.body.style.userSelect = "";
};

// Provide
provide("isRunning", isRunning);
provide("zoomLevel", zoomLevel);
</script>

<style scoped>
.app-container {
  display: flex;
  flex-direction: column;
  height: 100vh;
  width: 100vw;
  background: var(--surface-ground);
}

.main-content {
  display: flex;
  flex: 1;
  overflow: hidden;
}

.canvas-area {
  flex: 1;
  position: relative;
  background: var(--surface-section);
  overflow: hidden;
}

.resizer {
  width: 4px;
  background: transparent;
  cursor: col-resize;
  flex-shrink: 0;
  position: relative;
  z-index: 10;
  transition: background 0.2s ease;
}

.resizer:hover,
.resizer:active {
  background: var(--primary-color);
}

.resizer::before {
  content: "";
  position: absolute;
  top: 0;
  bottom: 0;
  left: -4px;
  right: -4px;
}

.resizer-left {
  margin-left: -2px;
}

.unsaved-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 8px 0;
  text-align: center;
}

.unsaved-content i {
  font-size: 2.5rem;
  color: var(--yellow-500);
}

.unsaved-content p {
  margin: 0;
  font-size: 0.9375rem;
  color: var(--text-color);
  line-height: 1.5;
}
</style>
