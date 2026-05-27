<template>
  <div class="topbar" data-tauri-drag-region @mousedown="onDragStart">
    <span class="topbar-title">{{ workflowName || '未命名工作流' }}</span>
    <div class="topbar-actions">
      <Button
        icon="pi pi-save"
        text
        size="small"
        v-tooltip.bottom="'保存'"
        @click="$emit('save')"
      />
      <Button
        icon="pi pi-cog"
        text
        size="small"
        v-tooltip.bottom="'设置'"
        @click="$emit('toggle-settings')"
      />
      <div class="status-indicator" :class="connectionStatus">
        <span class="status-dot"></span>
        <span class="status-text">{{ connectionStatusText }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import Button from "primevue/button";
import { getCurrentWindow } from "@tauri-apps/api/window";

defineProps<{
  workflowName: string;
  connectionStatus: "connected" | "idle";
  connectionStatusText: string;
}>();

defineEmits<{
  save: [];
  "toggle-settings": [];
}>();

function onDragStart(e: MouseEvent) {
  const target = e.target as HTMLElement;
  if (target.closest("button, .p-button")) return;
  getCurrentWindow().startDragging();
}
</script>

<style scoped>
.topbar {
  display: flex;
  align-items: center;
  height: 38px;
  padding: 0 12px 0 80px;
  flex-shrink: 0;
  user-select: none;
}

.topbar-title {
  flex: 1;
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--text-color);
  opacity: 0.8;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  pointer-events: none;
}

.topbar-actions {
  display: flex;
  align-items: center;
  gap: 4px;
  flex-shrink: 0;
}

.status-indicator {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 2px 8px;
  border-radius: 10px;
  font-size: 0.6875rem;
  margin-left: 8px;
}

.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--text-color-secondary);
}

.status-indicator.connected .status-dot {
  background: var(--green-400);
  box-shadow: 0 0 4px var(--green-400);
}

.status-indicator.idle .status-dot {
  background: var(--text-color-secondary);
}
</style>
