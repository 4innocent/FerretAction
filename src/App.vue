<script setup lang="ts">
import { useAutomation } from './composables/useAutomation'
import Header from './components/automation/Header.vue'
import LeftSidebar from './components/automation/LeftSidebar.vue'
import WorkflowCanvas from './components/automation/WorkflowCanvas.vue'
import NodeConfigPanel from './components/automation/NodeConfigPanel.vue'
import BottomPanel from './components/automation/BottomPanel.vue'
import WorkflowManager from './components/automation/WorkflowManager.vue'

// ============================================================
// PixelFlow — 桌面自动化工作流编辑器
// 布局：Header | LeftSidebar | Canvas+BottomPanel | NodeConfigPanel
// ============================================================

const {
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
} = useAutomation()
</script>

<template>
  <div class="flex h-screen flex-col overflow-hidden bg-background text-foreground">
    <!-- ====== 顶部 Header ====== -->
    <Header
      :workflow-name="currentWorkflow?.name || '未命名工作流'"
      :current-workflow="currentWorkflow"
      :recent-workflows="recentWorkflows"
      :is-running="isRunning"
      :is-paused="isPaused"
      :is-debug-mode="isDebugMode"
      :has-unsaved-changes="hasUnsavedChanges"
      @run="run"
      @pause="pause"
      @stop="stop"
      @step-forward="stepForward"
      @toggle-debug="toggleDebug"
      @save="save"
      @open-settings="() => {}"
      @open-workflow-manager="isWorkflowManagerOpen = true"
      @select-workflow="selectWorkflow"
      @new-workflow="createWorkflow('新建工作流')"
    />

    <!-- ====== 主体：三栏布局 ====== -->
    <div class="flex-1 min-h-0 flex">
      <!-- 左侧边栏 -->
      <div class="w-[20%] min-w-[200px] max-w-[300px] shrink-0">
        <LeftSidebar
          :image-targets="imageTargets"
          :selected-target-id="selectedTargetId"
          @select-target="selectedTargetId = $event"
          @add-target="addImageTarget"
          @delete-target="deleteImageTarget"
          @duplicate-target="duplicateImageTarget"
          @drag-node-start="(e, type) => { e.dataTransfer?.setData('nodeType', type) }"
        />
      </div>

      <!-- 中间：画布 + 底部面板 -->
      <div class="flex-1 min-w-0 flex flex-col">
        <div class="flex-1 min-h-0">
          <WorkflowCanvas
            :nodes="nodes"
            :connections="connections"
            :selected-node-id="selectedNodeId"
            @select-node="selectNode"
            @update-node="updateNode"
            @add-node="addNode"
          />
        </div>

        <BottomPanel
          :logs="logs"
          :selected-log-id="selectedLogId"
          :screenshot="screenshot"
          :match-results="matchResults"
          :is-searching="isSearching"
          :selected-match-id="selectedMatchId"
          @clear-logs="clearLogs"
          @export-logs="exportLogs"
          @select-log="selectedLogId = $event"
          @capture="capture"
          @search="search"
          @select-match="selectedMatchId = $event"
        />
      </div>

      <!-- 右侧：节点配置面板 -->
      <div class="w-[25%] min-w-[250px] max-w-[350px] shrink-0 border-l border-border bg-card">
        <NodeConfigPanel
          :node="selectedNode"
          @update="(updates) => selectedNodeId && updateNode(selectedNodeId, updates)"
          @delete="deleteSelectedNode"
          @duplicate="duplicateSelectedNode"
        />
      </div>
    </div>

    <!-- ====== 工作流管理器弹窗 ====== -->
    <WorkflowManager
      :workflows="workflows"
      :folders="folders"
      :current-workflow-id="currentWorkflowId"
      :is-open="isWorkflowManagerOpen"
      @close="isWorkflowManagerOpen = false"
      @select-workflow="selectWorkflow"
      @create-workflow="createWorkflow"
      @delete-workflow="deleteWorkflow"
      @duplicate-workflow="duplicateWorkflow"
      @rename-workflow="renameWorkflow"
      @run-workflow="runWorkflow"
      @toggle-favorite="toggleFavorite"
      @create-folder="createFolder"
      @delete-folder="deleteFolder"
      @rename-folder="renameFolder"
      @toggle-folder-expand="toggleFolderExpand"
      @move-to-folder="moveToFolder"
      @import-workflow="importWorkflow"
      @export-workflow="exportWorkflow"
    />
  </div>
</template>

<style>
/* ============================================================
   全局样式 — 暗色主题基础
   使用 UnoCSS 预设 + 自定义 CSS 变量
   ============================================================ */

:root {
  --background: oklch(0.13 0.005 260);
  --foreground: oklch(0.95 0.005 260);
  --card: oklch(0.15 0.005 260);
  --card-foreground: oklch(0.95 0.005 260);
  --primary: oklch(0.65 0.15 165);
  --primary-foreground: oklch(0.13 0.005 260);
  --secondary: oklch(0.2 0.005 260);
  --secondary-foreground: oklch(0.85 0.005 260);
  --muted: oklch(0.2 0.005 260);
  --muted-foreground: oklch(0.55 0.005 260);
  --border: oklch(0.25 0.005 260);
  --destructive: oklch(0.55 0.15 25);
  --destructive-foreground: oklch(0.95 0.005 260);
  --sidebar: oklch(0.11 0.005 260);
  --sidebar-foreground: oklch(0.85 0.005 260);
}

* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html, body, #app {
  height: 100%;
  overflow: hidden;
  font-family: 'Inter', system-ui, -apple-system, sans-serif;
  background-color: var(--background);
  color: var(--foreground);
}

/* 滚动条样式 */
::-webkit-scrollbar {
  width: 6px;
  height: 6px;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background: oklch(0.3 0.005 260);
  border-radius: 3px;
}

::-webkit-scrollbar-thumb:hover {
  background: oklch(0.4 0.005 260);
}
</style>
