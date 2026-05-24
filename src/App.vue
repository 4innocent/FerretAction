<template>
  <div class="app-container">
    <!-- Top Toolbar -->
    <header class="app-header">
      <div class="header-left">
        <div class="logo">
          <i
            class="pi pi-bolt"
            style="color: var(--primary-color); font-size: 1.25rem"
          ></i>
          <span class="logo-text">AutoVision</span>
        </div>
        <div class="workflow-name">
          <InputText
            v-model="workflowName"
            placeholder="未命名工作流"
            class="workflow-input"
          />
        </div>
      </div>

      <div class="header-center">
        <div class="execution-controls">
          <Button
            :icon="isRunning ? 'pi pi-stop' : 'pi pi-play'"
            :label="isRunning ? '停止' : '运行'"
            :severity="isRunning ? 'danger' : 'success'"
            @click="toggleExecution"
          />
          <Button
            icon="pi pi-step-forward"
            label="单步执行"
            severity="secondary"
            :disabled="isRunning"
          />
          <Button
            icon="pi pi-refresh"
            label="重置"
            severity="secondary"
            outlined
          />
        </div>
      </div>

      <div class="header-right">
        <Button icon="pi pi-save" v-tooltip="'保存 (Ctrl+S)'" text />
        <Button icon="pi pi-file-export" v-tooltip="'导出'" text />
        <Button
          icon="pi pi-cog"
          v-tooltip="'设置'"
          text
          @click="showSettings = true"
        />
        <div class="status-indicator" :class="connectionStatus">
          <span class="status-dot"></span>
          <span class="status-text">{{ connectionStatusText }}</span>
        </div>
      </div>
    </header>

    <!-- Main Content -->
    <div class="main-content">
      <!-- Left Sidebar -->
      <aside class="left-sidebar" :style="{ width: leftSidebarWidth + 'px' }">
        <Tabs v-model:value="leftTab">
          <TabList>
            <Tab value="workflows">
              <i class="pi pi-folder-open"></i>
              <span>工作流</span>
            </Tab>
            <Tab value="images">
              <i class="pi pi-image"></i>
              <span>图像目标</span>
            </Tab>
            <Tab value="actions">
              <i class="pi pi-box"></i>
              <span>操作块</span>
            </Tab>
          </TabList>
          <TabPanels>
            <TabPanel value="workflows">
              <WorkflowManager
                @select="onWorkflowSelect"
                @run="onWorkflowRun"
                @create="onWorkflowCreate"
              />
            </TabPanel>
            <TabPanel value="images">
              <ImageTargetLibrary
                :targets="imageTargets"
                @select="selectImageTarget"
                @add="addImageTarget"
                @delete="deleteImageTarget"
              />
            </TabPanel>
            <TabPanel value="actions">
              <ActionBlockPalette @drag-start="onDragStart" />
            </TabPanel>
          </TabPanels>
        </Tabs>
      </aside>

      <!-- Left Resizer -->
      <div class="resizer resizer-left" @mousedown="startResizeLeft"></div>

      <!-- Center Canvas Area -->
      <main class="canvas-area">
        <WorkflowCanvas
          ref="canvasRef"
          :nodes="workflowNodes"
          :edges="workflowEdges"
          :selected-node="selectedNode"
          @node-select="onNodeSelect"
          @node-add="onNodeAdd"
          @node-delete="onNodeDelete"
          @edge-connect="onEdgeConnect"
          @edge-delete="onEdgeDelete"
        />
      </main>

      <!-- Right Resizer -->
      <div class="resizer resizer-right" @mousedown="startResizeRight"></div>

      <!-- Right Panel -->
      <aside class="right-sidebar" :style="{ width: rightSidebarWidth + 'px' }">
        <div v-if="selectedNode" class="node-config-panel">
          <div class="panel-header">
            <div
              class="node-type-badge"
              :style="{ background: getNodeColor(selectedNode.type) }"
            >
              <i :class="getNodeIcon(selectedNode.type)"></i>
            </div>
            <span class="panel-title">{{
              getNodeLabel(selectedNode.type)
            }}</span>
            <Button
              icon="pi pi-times"
              text
              size="small"
              @click="selectedNode = null"
            />
          </div>

          <NodeConfigPanel
            :node="selectedNode"
            :image-targets="imageTargets"
            @update="updateNodeConfig"
          />
        </div>

        <div v-else class="empty-panel">
          <i class="pi pi-arrow-left"></i>
          <p>选择一个节点以查看配置</p>
        </div>
      </aside>
    </div>

    <!-- Bottom Panel -->
    <div class="bottom-panel" :class="{ expanded: bottomPanelExpanded }">
      <div
        class="panel-toggle"
        @click="bottomPanelExpanded = !bottomPanelExpanded"
      >
        <i
          :class="
            bottomPanelExpanded ? 'pi pi-chevron-down' : 'pi pi-chevron-up'
          "
        ></i>
        <span>执行日志 & 调试器</span>
        <span class="log-count" v-if="executionLogs.length">{{
          executionLogs.length
        }}</span>
      </div>

      <div class="panel-content" v-show="bottomPanelExpanded">
        <Tabs v-model:value="bottomTab">
          <TabList>
            <Tab value="logs">
              <i class="pi pi-list"></i>
              执行日志
            </Tab>
            <Tab value="debugger">
              <i class="pi pi-eye"></i>
              屏幕识别调试
            </Tab>
          </TabList>
          <TabPanels>
            <TabPanel value="logs">
              <ExecutionLogs :logs="executionLogs" @clear="clearLogs" />
            </TabPanel>
            <TabPanel value="debugger">
              <ScreenDebugger
                :current-screenshot="currentScreenshot"
                :detected-regions="detectedRegions"
                @capture="captureScreen"
                @test-recognition="testRecognition"
              />
            </TabPanel>
          </TabPanels>
        </Tabs>
      </div>
    </div>

    <!-- Settings Dialog -->
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
import { ref, computed, provide } from "vue";
import Button from "primevue/button";
import InputText from "primevue/inputtext";
import Tabs from "primevue/tabs";
import TabList from "primevue/tablist";
import Tab from "primevue/tab";
import TabPanels from "primevue/tabpanels";
import TabPanel from "primevue/tabpanel";
import Dialog from "primevue/dialog";
import Toast from "primevue/toast";
import { useToast } from "primevue/usetoast";

import ImageTargetLibrary from "./components/ImageTargetLibrary.vue";
import ActionBlockPalette from "./components/ActionBlockPalette.vue";
import WorkflowCanvas from "./components/WorkflowCanvas.vue";
import NodeConfigPanel from "./components/NodeConfigPanel.vue";
import ExecutionLogs from "./components/ExecutionLogs.vue";
import ScreenDebugger from "./components/ScreenDebugger.vue";
import ScreenshotHistory from "./components/ScreenshotHistory.vue";
import SettingsPanel from "./components/SettingsPanel.vue";
import WorkflowManager from "./components/WorkflowManager.vue";
import type {
  WorkflowNode,
  ImageTarget,
  ExecutionLog,
  Screenshot,
  DetectedRegion,
} from "./types";

const toast = useToast();

// Workflow State
const workflowName = ref("登录自动化流程");
const isRunning = ref(false);
const zoomLevel = ref(1);
const selectedNode = ref<WorkflowNode | null>(null);
const leftTab = ref("workflows");
const bottomTab = ref("logs");
const bottomPanelExpanded = ref(true);
const showSettings = ref(false);

// Sidebar Resizing
const leftSidebarWidth = ref(280);
const rightSidebarWidth = ref(320);
const isResizing = ref(false);
const resizeSide = ref<"left" | "right" | null>(null);

// Connection Status
const connectionStatus = computed(() =>
  isRunning.value ? "connected" : "idle",
);
const connectionStatusText = computed(() =>
  isRunning.value ? "运行中" : "就绪",
);

// Canvas ref
const canvasRef = ref();
const minimapRef = ref();

// Image Targets
const imageTargets = ref<ImageTarget[]>([
  {
    id: "1",
    name: "登录按钮",
    thumbnail: "",
    matchThreshold: 0.9,
    lastMatch: { x: 520, y: 380, confidence: 0.95 },
  },
  {
    id: "2",
    name: "用户名输入框",
    thumbnail: "",
    matchThreshold: 0.85,
    lastMatch: { x: 320, y: 200, confidence: 0.92 },
  },
  {
    id: "3",
    name: "密码输入框",
    thumbnail: "",
    matchThreshold: 0.85,
    lastMatch: { x: 320, y: 280, confidence: 0.89 },
  },
  {
    id: "4",
    name: "验证码图片",
    thumbnail: "",
    matchThreshold: 0.8,
    lastMatch: null,
  },
  {
    id: "5",
    name: "确认对话框",
    thumbnail: "",
    matchThreshold: 0.9,
    lastMatch: null,
  },
]);

// Workflow Nodes
const workflowNodes = ref<WorkflowNode[]>([]);
const workflowEdges = ref<{ source: string; target: string; label?: string }[]>([]);

// Execution Logs
const executionLogs = ref<ExecutionLog[]>([]);

// Screenshots
const currentScreenshot = ref("");
const screenshots = ref<Screenshot[]>([
  {
    id: "1",
    timestamp: new Date(Date.now() - 60000),
    image: "",
    label: "执行前",
  },
  {
    id: "2",
    timestamp: new Date(Date.now() - 30000),
    image: "",
    label: "步骤 3 完成",
  },
  { id: "3", timestamp: new Date(), image: "", label: "当前" },
]);

const detectedRegions = ref<DetectedRegion[]>([
  {
    id: "1",
    x: 520,
    y: 380,
    width: 100,
    height: 40,
    confidence: 0.95,
    targetName: "登录按钮",
  },
  {
    id: "2",
    x: 320,
    y: 200,
    width: 200,
    height: 32,
    confidence: 0.92,
    targetName: "用户名输入框",
  },
]);

// Methods
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

const selectImageTarget = (target: ImageTarget) => {
  toast.add({
    severity: "info",
    summary: "选中目标",
    detail: target.name,
    life: 2000,
  });
};

const addImageTarget = () => {
  const newTarget: ImageTarget = {
    id: String(Date.now()),
    name: "新图像目标",
    thumbnail: "",
    matchThreshold: 0.85,
    lastMatch: null,
  };
  imageTargets.value.push(newTarget);
};

const deleteImageTarget = (id: string) => {
  imageTargets.value = imageTargets.value.filter(t => t.id !== id);
};

// Workflow Management
const onWorkflowSelect = (workflow: { id: string; name: string }) => {
  workflowName.value = workflow.name;
  toast.add({
    severity: "info",
    summary: "已切换工作流",
    detail: workflow.name,
    life: 2000,
  });
};

const onWorkflowRun = (workflow: { id: string; name: string }) => {
  workflowName.value = workflow.name;
  isRunning.value = true;
  toast.add({
    severity: "success",
    summary: "开始执行",
    detail: `正在运行: ${workflow.name}`,
    life: 3000,
  });
};

const onWorkflowCreate = (data: { name: string; description?: string }) => {
  workflowName.value = data.name;
  workflowNodes.value = [
    { id: "start", type: "start", label: "开始", x: 100, y: 200, config: {} },
  ];
  workflowEdges.value = [];
  toast.add({
    severity: "success",
    summary: "已创建",
    detail: `新工作流: ${data.name}`,
    life: 2000,
  });
};

const onDragStart = (type: string) => {
  // Handle drag start for action blocks
};

const onNodeSelect = (node: WorkflowNode | null) => {
  selectedNode.value = node;
};

const onNodeAdd = (node: WorkflowNode) => {
  workflowNodes.value.push(node);
};

const onNodeDelete = (nodeId: string) => {
  workflowNodes.value = workflowNodes.value.filter(n => n.id !== nodeId);
  workflowEdges.value = workflowEdges.value.filter(
    e => e.source !== nodeId && e.target !== nodeId,
  );
  if (selectedNode.value?.id === nodeId) {
    selectedNode.value = null;
  }
};

const onEdgeConnect = (edge: { source: string; target: string }) => {
  workflowEdges.value.push(edge);
};

const onEdgeDelete = (edge: { source: string; target: string }) => {
  workflowEdges.value = workflowEdges.value.filter(
    (e) => !(e.source === edge.source && e.target === edge.target),
  );
};

const updateNodeConfig = (nodeId: string, config: Record<string, unknown>) => {
  const node = workflowNodes.value.find(n => n.id === nodeId);
  if (node) {
    node.config = { ...node.config, ...config };
  }
};

const clearLogs = () => {
  executionLogs.value = [];
};

const captureScreen = () => {
  toast.add({
    severity: "info",
    summary: "截图",
    detail: "正在在捕获屏幕...",
    life: 2000,
  });
};

const testRecognition = () => {
  toast.add({
    severity: "info",
    summary: "测试识别",
    detail: "正在测试图像识别...",
    life: 2000,
  });
};

const viewScreenshot = (screenshot: Screenshot) => {
  currentScreenshot.value = screenshot.image;
};

// Node helpers
const getNodeColor = (type: string): string => {
  const colors: Record<string, string> = {
    start: "#22c55e",
    "find-image": "#6366f1",
    "move-mouse": "#f59e0b",
    click: "#ef4444",
    "type-text": "#8b5cf6",
    wait: "#64748b",
    condition: "#06b6d4",
    loop: "#ec4899",
  };
  return colors[type] || "#6366f1";
};

const getNodeIcon = (type: string): string => {
  const icons: Record<string, string> = {
    start: "pi pi-play",
    "find-image": "pi pi-search",
    "move-mouse": "pi pi-arrows-alt",
    click: "pi pi-external-link",
    "type-text": "pi pi-pencil",
    wait: "pi pi-clock",
    condition: "pi pi-question-circle",
    loop: "pi pi-replay",
  };
  return icons[type] || "pi pi-box";
};

const getNodeLabel = (type: string): string => {
  const labels: Record<string, string> = {
    start: "开始节点",
    "find-image": "查找图像",
    "move-mouse": "移动鼠标",
    click: "点击",
    "type-text": "输入文本",
    wait: "等待",
    condition: "条件判断",
    loop: "循环",
  };
  return labels[type] || "未知节点";
};

// Provide global state
provide("isRunning", isRunning);
provide("zoomLevel", zoomLevel);

// Sidebar Resizing Methods
const startResizeLeft = (e: MouseEvent) => {
  isResizing.value = true;
  resizeSide.value = "left";
  document.addEventListener("mousemove", handleResize);
  document.addEventListener("mouseup", stopResize);
  document.body.style.cursor = "col-resize";
  document.body.style.userSelect = "none";
};

const startResizeRight = (e: MouseEvent) => {
  isResizing.value = true;
  resizeSide.value = "right";
  document.addEventListener("mousemove", handleResize);
  document.addEventListener("mouseup", stopResize);
  document.body.style.cursor = "col-resize";
  document.body.style.userSelect = "none";
};

const handleResize = (e: MouseEvent) => {
  if (!isResizing.value) return;

  if (resizeSide.value === "left") {
    const newWidth = e.clientX;
    leftSidebarWidth.value = Math.max(200, Math.min(500, newWidth));
  } else if (resizeSide.value === "right") {
    const newWidth = window.innerWidth - e.clientX;
    rightSidebarWidth.value = Math.max(250, Math.min(500, newWidth));
  }
};

const stopResize = () => {
  isResizing.value = false;
  resizeSide.value = null;
  document.removeEventListener("mousemove", handleResize);
  document.removeEventListener("mouseup", stopResize);
  document.body.style.cursor = "";
  document.body.style.userSelect = "";
};
</script>

<style scoped>
.app-container {
  display: flex;
  flex-direction: column;
  height: 100vh;
  width: 100vw;
  background: var(--surface-ground);
}

/* Header */
.app-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 56px;
  padding: 0 16px;
  background: var(--surface-section);
  border-bottom: 1px solid var(--surface-border);
  flex-shrink: 0;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 24px;
}

.logo {
  display: flex;
  align-items: center;
  gap: 8px;
}

.logo-text {
  font-size: 1.125rem;
  font-weight: 700;
  background: linear-gradient(135deg, var(--primary-color), var(--cyan-400));
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.workflow-input {
  width: 200px;
  background: transparent;
  border: 1px solid transparent;
}

.workflow-input:hover {
  border-color: var(--surface-border);
}

.header-center {
  display: flex;
  align-items: center;
}

.execution-controls {
  display: flex;
  gap: 8px;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.status-indicator {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 12px;
  background: var(--surface-card);
  border-radius: 20px;
  font-size: 0.75rem;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--text-color-secondary);
}

.status-indicator.connected .status-dot {
  background: var(--green-400);
  box-shadow: 0 0 8px var(--green-400);
}

.status-indicator.idle .status-dot {
  background: var(--text-color-secondary);
}

/* Main Content */
.main-content {
  display: flex;
  flex: 1;
  overflow: hidden;
}

/* Left Sidebar */
.left-sidebar {
  background: var(--surface-section);
  border-right: 1px solid var(--surface-border);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  min-width: 200px;
  max-width: 500px;
}

.left-sidebar :deep(.p-tabs) {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.left-sidebar :deep(.p-tablist) {
  background: transparent;
  padding: 8px;
}

.left-sidebar :deep(.p-tab) {
  flex: 1;
  justify-content: center;
  gap: 6px;
  padding: 8px;
  font-size: 0.8125rem;
  background: transparent;
  border: none;
  color: var(--text-color-secondary);
}

.left-sidebar :deep(.p-tab[data-p-active="true"]) {
  background: var(--surface-card);
  color: var(--text-color);
}

.left-sidebar :deep(.p-tabpanels) {
  flex: 1;
  overflow: hidden;
}

.left-sidebar :deep(.p-tabpanel) {
  height: 100%;
  padding: 0;
}

/* Resizer */
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

.resizer-right {
  margin-right: -2px;
}

/* Canvas Area */
.canvas-area {
  flex: 1;
  position: relative;
  background: var(--surface-section);
  overflow: hidden;
}

.canvas-toolbar {
  position: absolute;
  bottom: 16px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 8px;
  background: var(--surface-overlay);
  border: 1px solid var(--surface-border);
  border-radius: 8px;
}

.zoom-level {
  font-size: 0.75rem;
  color: var(--text-color-secondary);
  padding: 0 8px;
}

.minimap {
  position: absolute;
  bottom: 16px;
  right: 16px;
  width: 200px;
  height: 120px;
  background: var(--surface-overlay);
  border: 1px solid var(--surface-border);
  border-radius: 8px;
  overflow: hidden;
}

.minimap-content {
  width: 100%;
  height: 100%;
}

/* Right Sidebar */
.right-sidebar {
  background: var(--surface-section);
  border-left: 1px solid var(--surface-border);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  min-width: 250px;
  max-width: 500px;
}

.node-config-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.panel-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px;
  border-bottom: 1px solid var(--surface-border);
}

.node-type-badge {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 8px;
  color: white;
}

.panel-title {
  flex: 1;
  font-weight: 600;
}

.empty-panel {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--text-color-secondary);
  gap: 12px;
}

.empty-panel i {
  font-size: 2rem;
  opacity: 0.5;
}

/* Bottom Panel */
.bottom-panel {
  background: var(--surface-section);
  border-top: 1px solid var(--surface-border);
  transition: height 0.3s ease;
  flex-shrink: 0;
}

.bottom-panel:not(.expanded) {
  height: 40px;
}

.bottom-panel.expanded {
  height: 280px;
}

.panel-toggle {
  display: flex;
  align-items: center;
  gap: 8px;
  height: 40px;
  padding: 0 16px;
  cursor: pointer;
  font-size: 0.875rem;
  color: var(--text-color-secondary);
  border-bottom: 1px solid var(--surface-border);
}

.panel-toggle:hover {
  background: var(--surface-hover);
}

.log-count {
  background: var(--primary-color);
  color: white;
  font-size: 0.625rem;
  padding: 2px 6px;
  border-radius: 10px;
}

.panel-content {
  height: calc(100% - 40px);
  overflow: hidden;
}

.panel-content :deep(.p-tabs) {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.panel-content :deep(.p-tablist) {
  padding: 0 16px;
  background: transparent;
}

.panel-content :deep(.p-tab) {
  gap: 6px;
  padding: 8px 12px;
  font-size: 0.8125rem;
  background: transparent;
  border: none;
  color: var(--text-color-secondary);
}

.panel-content :deep(.p-tab[data-p-active="true"]) {
  color: var(--primary-color);
  border-bottom: 2px solid var(--primary-color);
}

.panel-content :deep(.p-tabpanels) {
  flex: 1;
  overflow: hidden;
}

.panel-content :deep(.p-tabpanel) {
  height: 100%;
  padding: 0;
}
</style>
