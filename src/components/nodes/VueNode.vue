<template>
  <div
    class="vue-node"
    :class="{ 'is-highlighted': highlighted, 'is-running': running }"
  >
    <div class="vue-node-header" :style="{ background: headerColor }">
      <span class="vue-node-icon">{{ icon }}</span>
      <span class="vue-node-type">{{ typeLabel }}</span>
    </div>
    <div v-if="hasNote" class="vue-node-note">
      <pre>{{ label }}</pre>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";

const props = defineProps<{ node: any }>();

interface NodeData {
  nodeType?: string;
  nodeConfig?: Record<string, unknown>;
  label?: string;
  highlighted?: boolean;
  running?: boolean;
}

const nodeData = ref<NodeData>({});

function syncData() {
  nodeData.value = (props.node?.getData() as NodeData) ?? {};
}

onMounted(() => {
  syncData();
  props.node?.on("change:data", syncData);
});

onUnmounted(() => {
  props.node?.off("change:data", syncData);
});

const nodeType = computed(() => nodeData.value.nodeType || "");
const label = computed(() => nodeData.value.label || "");
const highlighted = computed(() => nodeData.value.highlighted || false);
const running = computed(() => nodeData.value.running || false);
const hasNote = computed(() => label.value.trim() !== "");

const iconMap: Record<string, string> = {
  start: "▶",
  end: "■",
  condition: "?",
  loop: "↻",
  break: "⏏",
  "find-image": "⌕",
  "image-gone": "⊘",
  "move-mouse": "⇅",
  click: "☛",
  "double-click": "⨍",
  scroll: "↕",
  "type-text": "✎",
  hotkey: "⚡",
  "key-press": "◎",
  wait: "⏳",
  "wait-condition": "⧖",
};

const colorMap: Record<string, string> = {
  start: "#22c55e",
  end: "#22c55e",
  condition: "#ec4899",
  loop: "#ec4899",
  break: "#ec4899",
  "find-image": "#6366f1",
  "image-gone": "#6366f1",
  "move-mouse": "#f59e0b",
  click: "#f59e0b",
  "double-click": "#f59e0b",
  scroll: "#f59e0b",
  "type-text": "#8b5cf6",
  hotkey: "#8b5cf6",
  "key-press": "#8b5cf6",
  wait: "#64748b",
  "wait-condition": "#475569",
};

const typeLabelMap: Record<string, string> = {
  start: "开始",
  end: "结束",
  condition: "条件判断",
  loop: "循环",
  break: "跳出循环",
  "find-image": "查找图像",
  "image-gone": "图像消失",
  "move-mouse": "移动鼠标",
  click: "点击",
  "double-click": "双击",
  scroll: "滚动",
  "type-text": "输入文本",
  hotkey: "快捷键",
  "key-press": "按键",
  wait: "等待",
  "wait-condition": "等待条件",
};

const icon = computed(() => iconMap[nodeType.value] || "");
const headerColor = computed(() => colorMap[nodeType.value] || "#6366f1");
const typeLabel = computed(
  () => typeLabelMap[nodeType.value] || nodeType.value,
);
</script>

<style scoped>
.vue-node {
  width: 100%;
  height: 100%;
  border-radius: 10px;
  overflow: hidden;
  background: var(--surface-card);
  border: 1px solid var(--surface-border);
  display: flex;
  flex-direction: column;
  font-family: sans-serif;
  box-sizing: border-box;
}

.vue-node.is-highlighted {
  border-color: #6366f1;
  border-width: 2px;
}

.vue-node.is-running {
  border-color: #22d3ee;
  border-width: 2px;
}

.vue-node-header {
  height: 40px;
  min-height: 40px;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 12px;
  border-radius: 9px 9px 0 0;
  flex-shrink: 0;
}

.vue-node-icon {
  color: #fff;
  font-size: 13px;
  font-weight: 600;
  font-family: monospace;
  flex-shrink: 0;
}

.vue-node-type {
  color: #fff;
  font-size: 11px;
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.vue-node-note {
  flex: 1;
  overflow: hidden;
  padding: 4px 12px;
  min-height: 0;
}

.vue-node-note pre {
  margin: 0;
  padding: 0;
  font-family: sans-serif;
  font-size: 11px;
  line-height: 1.45;
  color: #a0a0b8;
  white-space: pre-wrap;
  word-break: break-word;
  height: 100%;
  overflow-y: auto;
}

.vue-node-note pre::-webkit-scrollbar {
  width: 4px;
}

.vue-node-note pre::-webkit-scrollbar-track {
  background: transparent;
}

.vue-node-note pre::-webkit-scrollbar-thumb {
  background: #3a3a4a;
  border-radius: 2px;
}
</style>
