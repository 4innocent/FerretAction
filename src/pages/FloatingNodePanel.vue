<template>
  <div v-if="selectedNode" class="floating-panel" :style="panelStyle">
    <div class="panel-header" @mousedown="startDrag">
      <div
        class="node-type-badge"
        :style="{ background: getNodeColor(selectedNode.type) }"
      >
        <i :class="getNodeIcon(selectedNode.type)"></i>
      </div>
      <span class="panel-title">{{ getNodeLabel(selectedNode.type) }}</span>
      <Button
        icon="pi pi-times"
        text
        size="small"
        @click="$emit('close')"
      />
    </div>

    <NodeConfigPanel
      :node="selectedNode"
      :image-targets="imageTargets"
      @update="(nodeId: string, config: Record<string, unknown>) => $emit('update-node-config', nodeId, config)"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onUnmounted } from "vue";
import Button from "primevue/button";
import NodeConfigPanel from "../components/NodeConfigPanel.vue";
import type { WorkflowNode, ImageTarget } from "../types";

defineProps<{
  selectedNode: WorkflowNode | null;
  imageTargets: ImageTarget[];
}>();

defineEmits<{
  "update-node-config": [nodeId: string, config: Record<string, unknown>];
  close: [];
}>();

const panelX = ref(0);
const panelY = ref(12);
const useRight = ref(true);

const panelStyle = computed(() => {
  if (useRight.value) {
    return { right: "16px", top: `${panelY.value}px` };
  }
  return { left: `${panelX.value}px`, top: `${panelY.value}px` };
});

const isDragging = ref(false);
let dragStartX = 0;
let dragStartY = 0;

const startDrag = (e: MouseEvent) => {
  if ((e.target as HTMLElement).closest("button, .p-button")) return;

  if (useRight.value) {
    const panelEl = (e.currentTarget as HTMLElement).parentElement;
    if (panelEl) {
      const rect = panelEl.getBoundingClientRect();
      const parentRect = panelEl.parentElement?.getBoundingClientRect();
      if (parentRect) {
        panelX.value = rect.left - parentRect.left;
        panelY.value = rect.top - parentRect.top;
      }
    }
    useRight.value = false;
  }

  isDragging.value = true;
  dragStartX = e.clientX - panelX.value;
  dragStartY = e.clientY - panelY.value;
  document.addEventListener("mousemove", onDrag);
  document.addEventListener("mouseup", stopDrag);
};

const onDrag = (e: MouseEvent) => {
  if (!isDragging.value) return;
  panelX.value = e.clientX - dragStartX;
  panelY.value = e.clientY - dragStartY;
};

const stopDrag = () => {
  isDragging.value = false;
  document.removeEventListener("mousemove", onDrag);
  document.removeEventListener("mouseup", stopDrag);
};

onUnmounted(() => {
  document.removeEventListener("mousemove", onDrag);
  document.removeEventListener("mouseup", stopDrag);
});

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
</script>

<style scoped>
.floating-panel {
  position: absolute;
  z-index: 100;
  width: 320px;
  max-height: calc(100% - 24px);
  background: var(--surface-section);
  border: 1px solid var(--surface-border);
  border-radius: 12px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4), 0 2px 8px rgba(0, 0, 0, 0.2);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  animation: panel-fade-in 0.2s ease;
}

@keyframes panel-fade-in {
  from {
    opacity: 0;
    transform: translateY(-8px) scale(0.98);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

.panel-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  border-bottom: 1px solid var(--surface-border);
  cursor: grab;
  user-select: none;
}

.panel-header:active {
  cursor: grabbing;
}

.node-type-badge {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 8px;
  color: white;
  flex-shrink: 0;
}

.panel-title {
  flex: 1;
  font-weight: 600;
  font-size: 0.875rem;
}

.floating-panel :deep(.node-config) {
  overflow-y: auto;
  max-height: calc(80vh - 65px);
}
</style>
