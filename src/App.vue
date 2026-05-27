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
          @update-node-config="updateNodeConfig"
          @close="selectedNode = null"
        />
      </main>
    </div>

    <BottomPanel />

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
        <Button label="不保存" text severity="danger" @click="handleUnsavedDiscard" />
        <Button label="取消" text @click="handleUnsavedCancel" />
        <Button label="保存" @click="handleUnsavedSave" />
      </template>
    </Dialog>

    <Dialog
      v-model:visible="showSettings"
      header="设置"
      :style="{ width: '480px' }"
      modal
      :contentStyle="{ padding: 0, overflow: 'hidden' }"
    >
      <SettingsPanel />
    </Dialog>

    <Toast />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, provide, watch, onMounted, onUnmounted, nextTick } from "vue";
import Dialog from "primevue/dialog";
import Toast from "primevue/toast";
import Button from "primevue/button";
import { useToast } from "primevue/usetoast";
import { useDebounceFn } from "@vueuse/core";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

import TopBar from "./pages/TopBar.vue";
import LeftSidebar from "./pages/LeftSidebar.vue";
import WorkflowCanvas from "./components/WorkflowCanvas.vue";
import FloatingNodePanel from "./pages/FloatingNodePanel.vue";
import BottomPanel from "./pages/BottomPanel.vue";
import SettingsPanel from "./components/SettingsPanel.vue";
import type { WorkflowNode, ImageTarget, Workflow } from "./types";
import { getWorkflow, saveWorkflowContent } from "./db";

const toast = useToast();

// Shared state
const workflowName = ref("");
const currentWorkflowId = ref<string | null>(null);
const isRunning = ref(false);
const zoomLevel = ref(1);
const selectedNode = ref<WorkflowNode | null>(null);
const showSettings = ref(false);

// Sidebar
const leftSidebarWidth = ref(280);
const isResizing = ref(false);
const canvasRef = ref();

// Connection Status
const connectionStatus = computed(() => (isRunning.value ? "connected" : "idle"));
const connectionStatusText = computed(() => (isRunning.value ? "运行中" : "就绪"));

// Image Targets (shared with FloatingNodePanel)
const imageTargets = ref<ImageTarget[]>([]);

// Workflow canvas
const workflowNodes = ref<WorkflowNode[]>([]);
const workflowEdges = ref<{ source: string; target: string; label?: string }[]>([]);
const isLoading = ref(false);
const isDirty = ref(false);
const showUnsavedDialog = ref(false);
const pendingWorkflow = ref<Workflow | null>(null);
// Snapshot of original state when workflow was first loaded / last saved
const originalNodes = ref<WorkflowNode[]>([]);
const originalEdges = ref<{ source: string; target: string; label?: string }[]>([]);

// ── Step execution ───────────────────────────────────────────

const ACTIONABLE_TYPES = new Set([
  "move-mouse", "click", "double-click", "drag", "scroll",
  "type-text", "hotkey", "key-press",
  "wait", "wait-condition",
]);

const selectedNodes = ref<{ id: string; type: string; config: Record<string, unknown> }[]>([]);
const canStepExecute = computed(() =>
  selectedNodes.value.length > 0 && selectedNodes.value.some(n => ACTIONABLE_TYPES.has(n.type)),
);

function onSelectionChange(nodes: { id: string; type: string; config: Record<string, unknown> }[]) {
  selectedNodes.value = nodes;
}

async function onStepExecute() {
  if (!canStepExecute.value) return;
  // Collect actionable nodes in selection order
  const nodes = selectedNodes.value.filter(n => ACTIONABLE_TYPES.has(n.type));
  try {
    // Minimize window first so automation doesn't interfere with the app
    await getCurrentWindow().minimize();
    // Small delay to let the minimize complete
    await new Promise(r => setTimeout(r, 300));
    await invoke("execute_step", { nodes });
  } catch (e: any) {
    console.error("Step execution failed:", e);
    toast.add({ severity: "error", summary: "执行失败", detail: String(e), life: 4000 });
  }
}

// Execution
const toggleExecution = () => {
  isRunning.value = !isRunning.value;
  if (isRunning.value) {
    toast.add({ severity: "info", summary: "开始执行", detail: "工作流已启动", life: 3000 });
  } else {
    toast.add({ severity: "warn", summary: "执行停止", detail: "工作流已停止", life: 3000 });
  }
};

// Image target mutations
const addImageTarget = (data: { name: string; thumbnail: string; folderId: string }) => {
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
  } catch (e) {
    console.error("Failed to load workflow:", e);
  } finally {
    await nextTick();
    isLoading.value = false;
  }
}

async function doSave() {
  if (!currentWorkflowId.value) return;
  await saveWorkflowContent(currentWorkflowId.value, workflowNodes.value, workflowEdges.value);
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
    await saveWorkflowContent(currentWorkflowId.value, workflowNodes.value, workflowEdges.value);
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

const onWorkflowRun = (workflow: Workflow) => {
  workflowName.value = workflow.name;
  isRunning.value = true;
  toast.add({ severity: "success", summary: "开始执行", detail: `正在运行: ${workflow.name}`, life: 3000 });
};

const onWorkflowCreate = async (data: { name: string; description?: string; folderId?: string }) => {
  workflowName.value = data.name;
  const startNode: WorkflowNode = { id: "start", type: "start", label: "开始", x: 100, y: 200, config: {} };
  workflowNodes.value = [startNode];
  workflowEdges.value = [];
  isDirty.value = true;
  snapshotState();
};

// ── Auto-save ────────────────────────────────────────────────

const debouncedSave = useDebounceFn(async () => {
  if (!currentWorkflowId.value) return;
  try {
    await saveWorkflowContent(currentWorkflowId.value, workflowNodes.value, workflowEdges.value);
  } catch (e) {
    console.error("Auto-save failed:", e);
  }
}, 500);

watch([workflowNodes, workflowEdges], () => {
  if (isLoading.value) return;
  if (currentWorkflowId.value) isDirty.value = true;
  debouncedSave();
}, { deep: true });

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

const onDragStart = (block: { type: string; label: string }, event: MouseEvent) => {
  canvasRef.value?.startDnd(block.type, block.label, event);
};

// Canvas events
const onNodeSelect = (node: WorkflowNode | null) => {
  selectedNode.value = node;
};

const onNodeAdd = (node: WorkflowNode) => {
  workflowNodes.value.push(node);
};

const onNodeDelete = (nodeId: string) => {
  workflowNodes.value = workflowNodes.value.filter(n => n.id !== nodeId);
  workflowEdges.value = workflowEdges.value.filter(e => e.source !== nodeId && e.target !== nodeId);
  if (selectedNode.value?.id === nodeId) selectedNode.value = null;
};

const onEdgeConnect = (edge: { source: string; target: string }) => {
  workflowEdges.value.push(edge);
};

const onEdgeDelete = (edge: { source: string; target: string }) => {
  workflowEdges.value = workflowEdges.value.filter(
    e => !(e.source === edge.source && e.target === edge.target),
  );
};

const updateNodeConfig = (nodeId: string, config: Record<string, unknown>) => {
  const node = workflowNodes.value.find(n => n.id === nodeId);
  if (node) node.config = { ...node.config, ...config };
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
