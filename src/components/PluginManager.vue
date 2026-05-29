<template>
  <div class="plugin-manager">
    <div class="plugin-toolbar">
      <span class="plugin-title">已安装插件</span>
      <Button icon="pi pi-plus" label="添加" size="small" severity="secondary" outlined />
    </div>
    <div class="plugin-list">
      <div class="plugin-card" v-for="plugin in plugins" :key="plugin.id">
        <div class="plugin-info">
          <span class="plugin-name">{{ plugin.name }}</span>
          <span class="plugin-version">v{{ plugin.version }}</span>
        </div>
        <Tag :value="plugin.enabled ? '已启用' : '已停用'" :severity="plugin.enabled ? 'success' : 'secondary'" />
      </div>
      <div v-if="plugins.length === 0" class="empty-plugins">
        <i class="pi pi-puzzle-piece"></i>
        <span>暂无安装插件</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import Button from "primevue/button";
import Tag from "primevue/tag";

interface Plugin {
  id: string;
  name: string;
  version: string;
  enabled: boolean;
}

const plugins = ref<Plugin[]>([]);
</script>

<style scoped>
.plugin-manager {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.plugin-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 16px;
  border-bottom: 1px solid var(--surface-border);
}

.plugin-title {
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--text-color);
}

.plugin-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px 16px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.plugin-card {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  background: var(--surface-card);
  border: 1px solid var(--surface-border);
  border-radius: 8px;
}

.plugin-info {
  display: flex;
  align-items: center;
  gap: 8px;
}

.plugin-name {
  font-size: 0.8125rem;
  font-weight: 500;
}

.plugin-version {
  font-size: 0.6875rem;
  color: var(--text-color-secondary);
}

.empty-plugins {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--text-color-secondary);
  gap: 8px;
  font-size: 0.8125rem;
}

.empty-plugins i {
  font-size: 1.5rem;
  opacity: 0.5;
}
</style>
