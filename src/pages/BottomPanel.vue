<template>
  <div class="bottom-panel" :class="{ expanded }">
    <div class="panel-toggle" @click="expanded = !expanded">
      <i :class="expanded ? 'pi pi-chevron-down' : 'pi pi-chevron-up'"></i>
      <span>执行日志 & 插件</span>
      <span class="log-count" v-if="executionLogs.length">{{
        executionLogs.length
      }}</span>
    </div>

    <div class="panel-content" v-show="expanded">
      <Tabs v-model:value="bottomTab">
        <TabList>
          <Tab value="logs">
            <i class="pi pi-list"></i>
            执行日志
          </Tab>
          <Tab value="plugins">
            <i class="pi pi-puzzle-piece"></i>
            插件管理
          </Tab>
        </TabList>
        <TabPanels>
          <TabPanel value="logs">
            <ExecutionLogs :logs="executionLogs" @clear="clearLogs" />
          </TabPanel>
          <TabPanel value="plugins">
            <PluginManager />
          </TabPanel>
        </TabPanels>
      </Tabs>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import Tabs from "primevue/tabs";
import TabList from "primevue/tablist";
import Tab from "primevue/tab";
import TabPanels from "primevue/tabpanels";
import TabPanel from "primevue/tabpanel";
import ExecutionLogs from "../components/ExecutionLogs.vue";
import PluginManager from "../components/PluginManager.vue";
import type { ExecutionLog } from "../types";

defineProps<{
  executionLogs: ExecutionLog[];
}>();

const emit = defineEmits<{
  clear: [];
}>();

const expanded = ref(false);
const bottomTab = ref("logs");

const clearLogs = () => {
  emit("clear");
};
</script>

<style scoped>
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
