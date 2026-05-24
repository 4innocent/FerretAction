<template>
  <div class="workflow-manager">
    <div class="manager-header">
      <h3>工作流管理</h3>
      <div class="header-actions">
        <Button
          icon="pi pi-plus"
          severity="success"
          size="small"
          @click="createWorkflow"
          v-tooltip.bottom="'新建工作流'"
        />
        <Button
          icon="pi pi-folder-plus"
          severity="secondary"
          size="small"
          @click="createFolder"
          v-tooltip.bottom="'新建文件夹'"
        />
        <Button
          icon="pi pi-upload"
          severity="secondary"
          size="small"
          @click="importWorkflow"
          v-tooltip.bottom="'导入工作流'"
        />
      </div>
    </div>

    <div class="workflow-stats">
      <div class="stat-item">
        <span class="stat-value">{{ workflows.length }}</span>
        <span class="stat-label">工作流</span>
      </div>
      <div class="stat-item">
        <span class="stat-value">{{ folders.length }}</span>
        <span class="stat-label">文件夹</span>
      </div>
      <div class="stat-item">
        <span class="stat-value">{{ runningCount }}</span>
        <span class="stat-label">运行中</span>
      </div>
    </div>

    <div class="workflow-tree">
      <Tree
        :value="treeNodes"
        v-model:selectionKeys="selectedKeys"
        selectionMode="single"
        :filter="true"
        filterMode="lenient"
        :filterPlaceholder="''"
        class="workflow-tree-component"
        @node-select="onNodeSelect"
        @node-unselect="onNodeUnselect"
      >
        <template #default="{ node }">
          <div
            class="tree-node"
            :class="{ 'is-active': node.data?.id === activeWorkflowId }"
          >
            <div class="node-icon">
              <i v-if="node.type === 'folder'" class="pi pi-folder" />
              <i v-else class="pi pi-sitemap" />
            </div>
            <div class="node-content">
              <span class="node-label">{{ node.label }}</span>
              <div v-if="node.type === 'workflow'" class="node-meta">
                <Tag
                  v-if="node.data?.status === 'running'"
                  severity="success"
                  value="运行中"
                  class="status-tag"
                />
                <Tag
                  v-else-if="node.data?.status === 'error'"
                  severity="danger"
                  value="错误"
                  class="status-tag"
                />
                <Tag
                  v-else-if="node.data?.status === 'paused'"
                  severity="warn"
                  value="暂停"
                  class="status-tag"
                />
                <span class="node-date">{{
                  formatDate(node.data?.updatedAt)
                }}</span>
              </div>
            </div>
            <div class="node-actions" @click.stop>
              <Button
                v-if="node.type === 'workflow'"
                icon="pi pi-play"
                severity="success"
                text
                size="small"
                @click="runWorkflow(node.data)"
                v-tooltip.bottom="'运行'"
              />
              <Button
                icon="pi pi-ellipsis-v"
                severity="secondary"
                text
                size="small"
                @click="showNodeMenu($event, node)"
              />
            </div>
          </div>
        </template>
      </Tree>
    </div>

    <Menu ref="nodeMenu" :model="nodeMenuItems" :popup="true" />

    <Dialog
      v-model:visible="showCreateDialog"
      :header="createDialogTitle"
      :modal="true"
      :style="{ width: '400px' }"
      class="create-dialog"
    >
      <div class="dialog-content">
        <div class="form-field">
          <label>名称</label>
          <InputText
            v-model="newItemName"
            placeholder="输入名称"
            class="w-full"
          />
        </div>
        <div v-if="createType === 'workflow'" class="form-field">
          <label>描述</label>
          <Textarea
            v-model="newItemDescription"
            placeholder="输入描述（可选）"
            rows="3"
            class="w-full"
          />
        </div>
        <div v-if="createType === 'workflow'" class="form-field">
          <label>保存位置</label>
          <Select
            v-model="selectedFolder"
            :options="folderOptions"
            optionLabel="label"
            optionValue="value"
            placeholder="选择文件夹"
            class="w-full"
          />
        </div>
        <div v-if="createType === 'workflow'" class="form-field">
          <label>模板</label>
          <Select
            v-model="selectedTemplate"
            :options="templateOptions"
            optionLabel="label"
            optionValue="value"
            placeholder="选择模板（可选）"
            class="w-full"
          />
        </div>
      </div>
      <template #footer>
        <Button
          label="取消"
          severity="secondary"
          @click="showCreateDialog = false"
        />
        <Button
          :label="createType === 'workflow' ? '创建工作流' : '创建文件夹'"
          @click="confirmCreate"
        />
      </template>
    </Dialog>

    <Dialog
      v-model:visible="showDeleteDialog"
      header="确认删除"
      :modal="true"
      :style="{ width: '400px' }"
      class="delete-dialog"
    >
      <div class="dialog-content">
        <i class="pi pi-exclamation-triangle warning-icon" />
        <p>
          确定要删除 <strong>{{ deleteTarget?.label }}</strong> 吗？
        </p>
        <p v-if="deleteTarget?.type === 'folder'" class="warning-text">
          文件夹内的所有工作流也将被删除！
        </p>
      </div>
      <template #footer>
        <Button
          label="取消"
          severity="secondary"
          @click="showDeleteDialog = false"
        />
        <Button label="删除" severity="danger" @click="confirmDelete" />
      </template>
    </Dialog>

    <Dialog
      v-model:visible="showRenameDialog"
      header="重命名"
      :modal="true"
      :style="{ width: '350px' }"
    >
      <div class="dialog-content">
        <div class="form-field">
          <label>新名称</label>
          <InputText
            v-model="renameValue"
            class="w-full"
            @keyup.enter="confirmRename"
          />
        </div>
      </div>
      <template #footer>
        <Button
          label="取消"
          severity="secondary"
          @click="showRenameDialog = false"
        />
        <Button label="确定" @click="confirmRename" />
      </template>
    </Dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import Button from "primevue/button";
import InputText from "primevue/inputtext";
import IconField from "primevue/iconfield";
import InputIcon from "primevue/inputicon";
import Tree from "primevue/tree";
import Tag from "primevue/tag";
import Menu from "primevue/menu";
import Dialog from "primevue/dialog";
import Textarea from "primevue/textarea";
import Select from "primevue/select";

interface Workflow {
  id: string;
  name: string;
  description?: string;
  folderId?: string;
  status: "idle" | "running" | "paused" | "error";
  createdAt: Date;
  updatedAt: Date;
  lastOpened?: Date;
  nodeCount: number;
}

interface Folder {
  id: string;
  name: string;
  parentId?: string;
  createdAt: Date;
}

const emit = defineEmits<{
  (e: "select", workflow: Workflow): void;
  (e: "run", workflow: Workflow): void;
  (
    e: "create",
    data: {
      name: string;
      description?: string;
      folderId?: string;
      template?: string;
    },
  ): void;
}>();

const searchQuery = ref("");
const selectedKeys = ref<Record<string, boolean>>({});
const activeWorkflowId = ref("wf-1");
const nodeMenu = ref();

const showCreateDialog = ref(false);
const createType = ref<"workflow" | "folder">("workflow");
const newItemName = ref("");
const newItemDescription = ref("");
const selectedFolder = ref<string | null>(null);
const selectedTemplate = ref<string | null>(null);

const showDeleteDialog = ref(false);
const deleteTarget = ref<{
  label: string;
  type: string;
  data?: Workflow | Folder;
} | null>(null);

const showRenameDialog = ref(false);
const renameValue = ref("");
const renameTarget = ref<{ key: string; type: string } | null>(null);

const folders = ref<Folder[]>([
  { id: "folder-1", name: "登录自动化", createdAt: new Date("2024-01-15") },
  { id: "folder-2", name: "数据采集", createdAt: new Date("2024-01-20") },
  { id: "folder-3", name: "测试脚本", createdAt: new Date("2024-02-01") },
]);

const workflows = ref<Workflow[]>([
  {
    id: "wf-1",
    name: "网站登录流程",
    description: "自动登录目标网站",
    folderId: "folder-1",
    status: "idle",
    createdAt: new Date("2024-01-15"),
    updatedAt: new Date("2024-03-10"),
    lastOpened: new Date("2024-03-10"),
    nodeCount: 8,
  },
  {
    id: "wf-2",
    name: "验证码识别登录",
    description: "带验证码的登录流程",
    folderId: "folder-1",
    status: "running",
    createdAt: new Date("2024-01-18"),
    updatedAt: new Date("2024-03-09"),
    lastOpened: new Date("2024-03-09"),
    nodeCount: 12,
  },
  {
    id: "wf-3",
    name: "商品信息采集",
    description: "采集电商平台商品数据",
    folderId: "folder-2",
    status: "idle",
    createdAt: new Date("2024-01-25"),
    updatedAt: new Date("2024-03-08"),
    lastOpened: new Date("2024-03-08"),
    nodeCount: 15,
  },
  {
    id: "wf-4",
    name: "价格监控",
    description: "定时监控商品价格变化",
    folderId: "folder-2",
    status: "paused",
    createdAt: new Date("2024-02-01"),
    updatedAt: new Date("2024-03-07"),
    nodeCount: 10,
  },
  {
    id: "wf-5",
    name: "UI回归测试",
    description: "自动化UI测试脚本",
    folderId: "folder-3",
    status: "error",
    createdAt: new Date("2024-02-10"),
    updatedAt: new Date("2024-03-06"),
    nodeCount: 20,
  },
  {
    id: "wf-6",
    name: "表单自动填充",
    description: "自动填写表单数据",
    status: "idle",
    createdAt: new Date("2024-02-15"),
    updatedAt: new Date("2024-03-05"),
    lastOpened: new Date("2024-03-05"),
    nodeCount: 6,
  },
  {
    id: "wf-7",
    name: "批量文件处理",
    description: "批量处理文件操作",
    status: "idle",
    createdAt: new Date("2024-02-20"),
    updatedAt: new Date("2024-03-04"),
    nodeCount: 9,
  },
]);

const runningCount = computed(() => {
  return workflows.value.filter(w => w.status === "running").length;
});

const treeNodes = computed(() => {
  const nodes: any[] = [];

  // Add folders with their workflows
  folders.value.forEach(folder => {
    const folderWorkflows = workflows.value.filter(
      w => w.folderId === folder.id,
    );
    nodes.push({
      key: folder.id,
      label: folder.name,
      type: "folder",
      data: folder,
      children: folderWorkflows.map(w => ({
        key: w.id,
        label: w.name,
        type: "workflow",
        data: w,
      })),
    });
  });

  // Add workflows without folder
  const rootWorkflows = workflows.value.filter(w => !w.folderId);
  rootWorkflows.forEach(w => {
    nodes.push({
      key: w.id,
      label: w.name,
      type: "workflow",
      data: w,
    });
  });

  return nodes;
});

const createDialogTitle = computed(() => {
  return createType.value === "workflow" ? "新建工作流" : "新建文件夹";
});

const folderOptions = computed(() => {
  return [
    { label: "根目录", value: null },
    ...folders.value.map(f => ({ label: f.name, value: f.id })),
  ];
});

const templateOptions = [
  { label: "空白工作流", value: null },
  { label: "网站登录模板", value: "login" },
  { label: "数据采集模板", value: "scraping" },
  { label: "表单填写模板", value: "form" },
  { label: "文件处理模板", value: "file" },
];

const nodeMenuItems = ref([
  {
    label: "打开",
    icon: "pi pi-folder-open",
    command: () => openSelectedNode(),
  },
  {
    label: "运行",
    icon: "pi pi-play",
    command: () => runSelectedNode(),
  },
  { separator: true },
  {
    label: "重命名",
    icon: "pi pi-pencil",
    command: () => renameSelectedNode(),
  },
  {
    label: "复制",
    icon: "pi pi-copy",
    command: () => duplicateSelectedNode(),
  },
  {
    label: "移动到...",
    icon: "pi pi-arrow-right",
    command: () => moveSelectedNode(),
  },
  { separator: true },
  {
    label: "导出",
    icon: "pi pi-download",
    command: () => exportSelectedNode(),
  },
  { separator: true },
  {
    label: "删除",
    icon: "pi pi-trash",
    class: "danger-item",
    command: () => deleteSelectedNode(),
  },
]);

let currentMenuNode: any = null;

function formatDate(date?: Date): string {
  if (!date) return "";
  return new Date(date).toLocaleDateString("zh-CN", {
    month: "short",
    day: "numeric",
  });
}

function createWorkflow() {
  createType.value = "workflow";
  newItemName.value = "";
  newItemDescription.value = "";
  selectedFolder.value = null;
  selectedTemplate.value = null;
  showCreateDialog.value = true;
}

function createFolder() {
  createType.value = "folder";
  newItemName.value = "";
  showCreateDialog.value = true;
}

function importWorkflow() {
  // Trigger file input for import
}

function confirmCreate() {
  if (!newItemName.value.trim()) return;

  if (createType.value === "workflow") {
    const newWorkflow: Workflow = {
      id: `wf-${Date.now()}`,
      name: newItemName.value,
      description: newItemDescription.value,
      folderId: selectedFolder.value || undefined,
      status: "idle",
      createdAt: new Date(),
      updatedAt: new Date(),
      lastOpened: new Date(),
      nodeCount: 0,
    };
    workflows.value.push(newWorkflow);
    emit("create", {
      name: newItemName.value,
      description: newItemDescription.value,
      folderId: selectedFolder.value || undefined,
      template: selectedTemplate.value || undefined,
    });
  } else {
    const newFolder: Folder = {
      id: `folder-${Date.now()}`,
      name: newItemName.value,
      createdAt: new Date(),
    };
    folders.value.push(newFolder);
  }

  showCreateDialog.value = false;
}

function onNodeSelect(node: any) {
  if (node.type === "workflow") {
    selectWorkflow(node.data);
  }
}

function onNodeUnselect() {
  // Handle unselect
}

function selectWorkflow(workflow: Workflow) {
  activeWorkflowId.value = workflow.id;
  workflow.lastOpened = new Date();
  emit("select", workflow);
}

function runWorkflow(workflow: Workflow) {
  emit("run", workflow);
}

function showNodeMenu(event: Event, node: any) {
  currentMenuNode = node;
  nodeMenu.value.toggle(event);
}

function openSelectedNode() {
  if (currentMenuNode?.type === "workflow") {
    selectWorkflow(currentMenuNode.data);
  }
}

function runSelectedNode() {
  if (currentMenuNode?.type === "workflow") {
    runWorkflow(currentMenuNode.data);
  }
}

function renameSelectedNode() {
  if (currentMenuNode) {
    renameTarget.value = {
      key: currentMenuNode.key,
      type: currentMenuNode.type,
    };
    renameValue.value = currentMenuNode.label;
    showRenameDialog.value = true;
  }
}

function confirmRename() {
  if (!renameTarget.value || !renameValue.value.trim()) return;

  if (renameTarget.value.type === "workflow") {
    const workflow = workflows.value.find(
      w => w.id === renameTarget.value?.key,
    );
    if (workflow) {
      workflow.name = renameValue.value;
      workflow.updatedAt = new Date();
    }
  } else {
    const folder = folders.value.find(f => f.id === renameTarget.value?.key);
    if (folder) {
      folder.name = renameValue.value;
    }
  }

  showRenameDialog.value = false;
  renameTarget.value = null;
}

function duplicateSelectedNode() {
  if (currentMenuNode?.type === "workflow") {
    const original = currentMenuNode.data as Workflow;
    const duplicate: Workflow = {
      ...original,
      id: `wf-${Date.now()}`,
      name: `${original.name} (副本)`,
      createdAt: new Date(),
      updatedAt: new Date(),
      status: "idle",
    };
    workflows.value.push(duplicate);
  }
}

function moveSelectedNode() {
  // Show move dialog
}

function exportSelectedNode() {
  // Export workflow as JSON
}

function deleteSelectedNode() {
  if (currentMenuNode) {
    deleteTarget.value = {
      label: currentMenuNode.label,
      type: currentMenuNode.type,
      data: currentMenuNode.data,
    };
    showDeleteDialog.value = true;
  }
}

function confirmDelete() {
  if (!deleteTarget.value) return;

  if (deleteTarget.value.type === "workflow") {
    const index = workflows.value.findIndex(
      w => w.id === (deleteTarget.value?.data as Workflow)?.id,
    );
    if (index > -1) {
      workflows.value.splice(index, 1);
    }
  } else {
    const folderId = (deleteTarget.value.data as Folder)?.id;
    // Remove folder
    const folderIndex = folders.value.findIndex(f => f.id === folderId);
    if (folderIndex > -1) {
      folders.value.splice(folderIndex, 1);
    }
    // Remove workflows in folder
    workflows.value = workflows.value.filter(w => w.folderId !== folderId);
  }

  showDeleteDialog.value = false;
  deleteTarget.value = null;
}
</script>

<style scoped>
.workflow-manager {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--surface-ground);
}

.manager-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid var(--surface-border);
}

.manager-header h3 {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  color: var(--text-color);
}

.header-actions {
  display: flex;
  gap: 4px;
}

.search-bar {
  padding: 12px 16px;
}

.search-bar :deep(.p-inputtext) {
  width: 100%;
  background: var(--surface-card);
  border-color: var(--surface-border);
}

.workflow-stats {
  display: flex;
  gap: 16px;
  padding: 8px 16px 16px;
}

.stat-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  flex: 1;
  padding: 8px;
  background: var(--surface-card);
  border-radius: 6px;
  border: 1px solid var(--surface-border);
}

.stat-value {
  font-size: 18px;
  font-weight: 600;
  color: var(--primary-color);
}

.stat-label {
  font-size: 11px;
  color: var(--text-color-secondary);
  margin-top: 2px;
}

.workflow-tree {
  flex: 1;
  overflow-y: auto;
  padding: 0 8px;
}

.workflow-tree-component {
  background: transparent;
  border: none;
  padding: 0;
}

.workflow-tree-component :deep(.p-tree-node) {
  padding: 0;
}

.workflow-tree-component :deep(.p-tree-node-content) {
  padding: 4px 8px;
  border-radius: 6px;
  transition: background 0.15s;
}

.workflow-tree-component :deep(.p-tree-node-content:hover) {
  background: var(--surface-hover);
}

.workflow-tree-component :deep(.p-tree-node-content.p-highlight) {
  background: var(--primary-color);
  background-opacity: 0.1;
}

.tree-node {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 4px 0;
}

.tree-node.is-active {
  color: var(--primary-color);
}

.node-icon {
  width: 20px;
  text-align: center;
  color: var(--text-color-secondary);
}

.tree-node.is-active .node-icon {
  color: var(--primary-color);
}

.node-content {
  flex: 1;
  min-width: 0;
}

.node-label {
  font-size: 13px;
  font-weight: 500;
  display: block;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.node-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 2px;
}

.status-tag {
  font-size: 10px;
  padding: 2px 6px;
}

.node-date {
  font-size: 11px;
  color: var(--text-color-secondary);
}

.node-actions {
  display: flex;
  gap: 2px;
  opacity: 0;
  transition: opacity 0.15s;
}

.tree-node:hover .node-actions {
  opacity: 1;
}

.dialog-content {
  padding: 8px 0;
}

.form-field {
  margin-bottom: 16px;
}

.form-field label {
  display: block;
  font-size: 13px;
  font-weight: 500;
  margin-bottom: 6px;
  color: var(--text-color);
}

.form-field :deep(.p-inputtext),
.form-field :deep(.p-textarea),
.form-field :deep(.p-select) {
  width: 100%;
}

.delete-dialog .dialog-content {
  text-align: center;
}

.warning-icon {
  font-size: 48px;
  color: var(--yellow-500);
  margin-bottom: 16px;
}

.warning-text {
  color: var(--red-400);
  font-size: 13px;
}

:deep(.danger-item) {
  color: var(--red-400) !important;
}

:deep(.danger-item .p-menuitem-icon) {
  color: var(--red-400) !important;
}

.w-full {
  width: 100%;
}
</style>
