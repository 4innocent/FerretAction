<template>
  <aside class="left-sidebar" :style="{ width: sidebarWidth + 'px' }">
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
            :active-workflow-id="activeWorkflowId"
            :sidebar-width="sidebarWidth"
            @select="$emit('workflow-select', $event)"
            @run="$emit('workflow-run', $event)"
            @create="$emit('workflow-create', $event)"
            @delete="$emit('workflow-delete', $event)"
          />
        </TabPanel>
        <TabPanel value="images">
          <ImageTargetLibrary
            :targets="imageTargets"
            :folders="imageFolders"
            :sidebar-width="sidebarWidth"
            @select="selectImageTarget"
            @add="$emit('image-add', $event)"
            @delete="$emit('image-delete', $event)"
            @add-folder="addFolder"
            @delete-folder="deleteFolder"
            @rename-folder="renameFolder"
            @rename-target="$emit('target-rename', $event[0], $event[1])"
          />
        </TabPanel>
        <TabPanel value="actions">
          <ActionBlockPalette @drag-start="(block, event) => $emit('drag-start', block, event)" />
        </TabPanel>
      </TabPanels>
    </Tabs>
  </aside>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { useToast } from "primevue/usetoast";
import Tabs from "primevue/tabs";
import TabList from "primevue/tablist";
import Tab from "primevue/tab";
import TabPanels from "primevue/tabpanels";
import TabPanel from "primevue/tabpanel";
import WorkflowManager from "../components/WorkflowManager.vue";
import ImageTargetLibrary from "../components/ImageTargetLibrary.vue";
import ActionBlockPalette from "../components/ActionBlockPalette.vue";
import type { ImageTarget, ImageFolder, Workflow } from "../types";

const toast = useToast();

defineProps<{
  sidebarWidth: number;
  imageTargets: ImageTarget[];
  activeWorkflowId: string | null;
}>();

defineEmits<{
  "workflow-select": [workflow: Workflow];
  "workflow-run": [workflow: Workflow];
  "workflow-create": [data: { name: string; description?: string; folderId?: string }];
  "workflow-delete": [workflowId: string];
  "image-add": [data: { name: string; thumbnail: string; folderId: string }];
  "image-delete": [id: string];
  "target-rename": [id: string, name: string];
  "drag-start": [block: { type: string; label: string }, event: MouseEvent];
}>();

const leftTab = ref("workflows");
const imageFolders = ref<ImageFolder[]>([{ id: "default", name: "默认文件夹" }]);

const selectImageTarget = (target: ImageTarget) => {
  toast.add({
    severity: "info",
    summary: "选中目标",
    detail: target.name,
    life: 2000,
  });
};

const addFolder = (name: string) => {
  imageFolders.value.push({ id: String(Date.now()), name });
};

const deleteFolder = (id: string) => {
  imageFolders.value = imageFolders.value.filter(f => f.id !== id);
};

const renameFolder = (id: string, name: string) => {
  const folder = imageFolders.value.find(f => f.id === id);
  if (folder) folder.name = name;
};
</script>

<style scoped>
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
</style>
