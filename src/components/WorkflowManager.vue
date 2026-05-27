<template>
  <div class="workflow-manager">
    <div class="manager-header">
      <div class="search-box" :style="{ maxWidth: Math.min(Math.max(100, sidebarWidth - 90), 320) + 'px' }">
        <i class="pi pi-search"></i>
        <InputText v-model="searchQuery" placeholder="搜索..." />
      </div>
      <Button
        icon="pi pi-folder-plus"
        size="small"
        @click="startCreateFolder"
        v-tooltip="'新建文件夹'"
      />
      <Button
        icon="pi pi-plus"
        size="small"
        severity="success"
        @click="startCreateWorkflow(null)"
        v-tooltip="'新建工作流'"
      />
    </div>

    <div class="folders-list">
      <!-- Root-level workflows (no folder) -->
      <div v-if="rootWorkflows.length > 0 || !searchQuery" class="folder-group">
        <div
          v-for="wf in rootWorkflows"
          :key="wf.id"
          class="workflow-row"
          :class="{ selected: activeWorkflowId === wf.id }"
          @click="selectWorkflow(wf)"
        >
          <div class="row-icon">
            <i class="pi pi-sitemap"></i>
          </div>
          <div class="row-content">
            <span class="row-name">{{ wf.name }}</span>
          </div>
          <div class="row-actions">
            <Button
              icon="pi pi-play"
              text
              size="small"
              class="row-action-btn"
              @click.stop="runWorkflow(wf)"
              v-tooltip="'运行'"
            />
            <Button
              icon="pi pi-pencil"
              text
              size="small"
              class="row-action-btn"
              @click.stop="startRenameWorkflow(wf)"
              v-tooltip="'重命名'"
            />
            <Button
              icon="pi pi-trash"
              text
              size="small"
              severity="danger"
              class="row-action-btn"
              @click.stop="confirmDeleteWorkflow(wf)"
              v-tooltip="'删除'"
            />
          </div>
        </div>
      </div>

      <!-- Folders -->
      <div
        v-for="folder in filteredFolders"
        :key="folder.id"
        class="folder-group"
      >
        <div class="folder-header" @click="toggleFolder(folder.id)">
          <i
            class="pi folder-chevron"
            :class="
              expandedFolders.has(folder.id)
                ? 'pi-chevron-down'
                : 'pi-chevron-right'
            "
          ></i>
          <i class="pi pi-folder-open folder-icon"></i>
          <span class="folder-name">{{ folder.name }}</span>
          <span class="folder-count">{{
            getFolderWorkflows(folder.id).length
          }}</span>
          <div class="folder-more" @click.stop>
            <Button
              icon="pi pi-ellipsis"
              text
              size="small"
              @click="toggleMenu(folder.id)"
            />
            <div v-if="activeMenu === folder.id" class="popup-menu">
              <button
                class="popup-item"
                @click="
                  startCreateWorkflow(folder.id);
                  activeMenu = null;
                "
              >
                <i class="pi pi-plus"></i>
                <span>新建工作流</span>
              </button>
              <button
                class="popup-item"
                @click="
                  startRenameFolder(folder);
                  activeMenu = null;
                "
              >
                <i class="pi pi-pencil"></i>
                <span>重命名</span>
              </button>
              <button
                class="popup-item popup-item-danger"
                @click="
                  confirmDeleteFolder(folder);
                  activeMenu = null;
                "
              >
                <i class="pi pi-trash"></i>
                <span>删除文件夹</span>
              </button>
            </div>
          </div>
        </div>

        <div v-if="expandedFolders.has(folder.id)" class="folder-workflows">
          <div
            v-for="wf in getFolderWorkflows(folder.id)"
            :key="wf.id"
            class="workflow-row"
            :class="{ selected: activeWorkflowId === wf.id }"
            @click="selectWorkflow(wf)"
          >
            <div class="row-icon">
              <i class="pi pi-sitemap"></i>
            </div>
            <div class="row-content">
              <span class="row-name">{{ wf.name }}</span>
            </div>
            <div class="row-actions">
              <Button
                icon="pi pi-play"
                text
                size="small"
                class="row-action-btn"
                @click.stop="runWorkflow(wf)"
                v-tooltip="'运行'"
              />
              <Button
                icon="pi pi-pencil"
                text
                size="small"
                class="row-action-btn"
                @click.stop="startRenameWorkflow(wf)"
                v-tooltip="'重命名'"
              />
              <Button
                icon="pi pi-trash"
                text
                size="small"
                severity="danger"
                class="row-action-btn"
                @click.stop="confirmDeleteWorkflow(wf)"
                v-tooltip="'删除'"
              />
            </div>
          </div>

          <div
            v-if="getFolderWorkflows(folder.id).length === 0"
            class="empty-folder"
          >
            <span>文件夹为空</span>
          </div>
        </div>
      </div>

      <div
        v-if="folders.length === 0 && workflows.length === 0 && !loading"
        class="empty-state"
      >
        <i class="pi pi-folder-open"></i>
        <p>暂无工作流</p>
        <Button
          label="新建工作流"
          icon="pi pi-plus"
          size="small"
          @click="startCreateWorkflow(null)"
        />
      </div>
    </div>

    <!-- Rename / Create Folder Dialog -->
    <Dialog
      v-model:visible="showRenameDialog"
      :header="renameTarget ? '重命名' : '新建文件夹'"
      :modal="true"
      :style="{ width: '350px' }"
    >
      <div class="dialog-body">
        <InputText
          v-model="renameValue"
          :placeholder="renameTarget ? '输入名称' : '文件夹名称'"
          class="w-full"
          @keyup.enter="confirmRename"
        />
      </div>
      <template #footer>
        <Button label="取消" text @click="showRenameDialog = false" />
        <Button
          :label="renameTarget ? '确定' : '创建'"
          @click="confirmRename"
        />
      </template>
    </Dialog>

    <!-- Create Dialog -->
    <Dialog
      v-model:visible="showCreateDialog"
      header="新建工作流"
      :modal="true"
      :style="{ width: '400px' }"
    >
      <div class="dialog-body">
        <div class="form-field">
          <label>名称</label>
          <InputText
            v-model="newItemName"
            placeholder="输入名称"
            class="w-full"
            @keyup.enter="confirmCreate"
          />
        </div>
        <div class="form-field">
          <label>描述</label>
          <Textarea
            v-model="newItemDescription"
            placeholder="输入描述（可选）"
            rows="3"
            class="w-full"
          />
        </div>
      </div>
      <template #footer>
        <Button label="取消" text @click="showCreateDialog = false" />
        <Button label="创建工作流" @click="confirmCreate" />
      </template>
    </Dialog>

    <!-- Delete Confirm Dialog -->
    <Dialog
      v-model:visible="showDeleteDialog"
      header="确认删除"
      :modal="true"
      :style="{ width: '400px' }"
    >
      <div class="dialog-body delete-content">
        <i class="pi pi-exclamation-triangle warning-icon" />
        <p>
          确定要删除 <strong>{{ deleteTargetName }}</strong> 吗？
        </p>
        <p v-if="deleteTargetType === 'folder'" class="warning-text">
          文件夹内的所有工作流也将被删除！
        </p>
      </div>
      <template #footer>
        <Button label="取消" text @click="showDeleteDialog = false" />
        <Button label="删除" severity="danger" @click="confirmDelete" />
      </template>
    </Dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import Button from "primevue/button";
import InputText from "primevue/inputtext";
import Dialog from "primevue/dialog";
import Textarea from "primevue/textarea";
import { useToast } from "primevue/usetoast";
import type { Workflow, Folder } from "../types";
import {
  initDb,
  listWorkflows,
  listFolders,
  createWorkflow as dbCreateWorkflow,
  createFolder as dbCreateFolder,
  deleteWorkflow as dbDeleteWorkflow,
  deleteFolder as dbDeleteFolder,
  updateWorkflow as dbUpdateWorkflow,
  renameFolder as dbRenameFolder,
} from "../db";

const toast = useToast();

const props = defineProps<{
  activeWorkflowId: string | null;
  sidebarWidth: number;
}>();

const emit = defineEmits<{
  (e: "select", workflow: Workflow): void;
  (e: "run", workflow: Workflow): void;
  (
    e: "create",
    data: { name: string; description?: string; folderId?: string },
  ): void;
  (e: "delete", workflowId: string): void;
}>();

// State
const searchQuery = ref("");
const expandedFolders = ref<Set<string>>(new Set());
const activeMenu = ref<string | null>(null);
const loading = ref(true);

const folders = ref<Folder[]>([]);
const workflows = ref<Workflow[]>([]);

// Dialog state
const showCreateDialog = ref(false);
const newItemName = ref("");
const newItemDescription = ref("");
const pendingFolderId = ref<string | null>(null);

const showRenameDialog = ref(false);
const renameValue = ref("");
const renameTarget = ref<{ id: string; type: "workflow" | "folder" } | null>(
  null,
);

const showDeleteDialog = ref(false);
const deleteTargetId = ref<string | null>(null);
const deleteTargetName = ref("");
const deleteTargetType = ref<"workflow" | "folder">("workflow");

// Computed
const rootWorkflows = computed(() => {
  if (searchQuery.value) {
    const q = searchQuery.value.toLowerCase();
    return workflows.value.filter(
      w => !w.folderId && w.name.toLowerCase().includes(q),
    );
  }
  return workflows.value.filter(w => !w.folderId);
});

const filteredFolders = computed(() => {
  if (!searchQuery.value) return folders.value;
  const q = searchQuery.value.toLowerCase();
  return folders.value.filter(f => {
    if (f.name.toLowerCase().includes(q)) return true;
    return getFolderWorkflows(f.id).some(w => w.name.toLowerCase().includes(q));
  });
});

function getFolderWorkflows(folderId: string): Workflow[] {
  return workflows.value.filter(w => w.folderId === folderId);
}

// Lifecycle
onMounted(async () => {
  try {
    await initDb();
    const [flds, wfs] = await Promise.all([listFolders(), listWorkflows()]);
    folders.value = flds;
    workflows.value = wfs;
  } catch (e: any) {
    toast.add({
      severity: "error",
      summary: "数据库初始化失败",
      detail: String(e),
      life: 6000,
    });
  } finally {
    loading.value = false;
  }
});

// Close popup menu on outside click
const closeMenu = (e: MouseEvent) => {
  const target = e.target as HTMLElement;
  if (!target.closest(".folder-more")) {
    activeMenu.value = null;
  }
};
onMounted(() => document.addEventListener("click", closeMenu));
onUnmounted(() => document.removeEventListener("click", closeMenu));

// Folder toggle
function toggleFolder(folderId: string) {
  if (expandedFolders.value.has(folderId)) {
    expandedFolders.value.delete(folderId);
  } else {
    expandedFolders.value.add(folderId);
  }
}

function toggleMenu(folderId: string) {
  activeMenu.value = activeMenu.value === folderId ? null : folderId;
}

// Format
function formatDate(date?: string): string {
  if (!date) return "";
  return new Date(date).toLocaleDateString("zh-CN", {
    month: "short",
    day: "numeric",
  });
}

// Create workflow
function startCreateWorkflow(folderId: string | null) {
  pendingFolderId.value = folderId;
  newItemName.value = "";
  newItemDescription.value = "";
  showCreateDialog.value = true;
}

async function confirmCreate() {
  if (!newItemName.value.trim()) return;
  try {
    const wf = await dbCreateWorkflow(
      newItemName.value,
      newItemDescription.value,
      pendingFolderId.value || undefined,
    );
    workflows.value.push(wf);
    emit("create", {
      name: newItemName.value,
      description: newItemDescription.value,
      folderId: pendingFolderId.value || undefined,
    });
    selectWorkflow(wf);
    showCreateDialog.value = false;
    toast.add({
      severity: "success",
      summary: "已创建",
      detail: wf.name,
      life: 2000,
    });
  } catch (e: any) {
    toast.add({
      severity: "error",
      summary: "创建失败",
      detail: String(e),
      life: 6000,
    });
  }
}

// Create folder - use rename dialog with renameTarget=null to signal creation
function startCreateFolder() {
  renameTarget.value = null;
  renameValue.value = "";
  showRenameDialog.value = true;
}

// Select / Run
function selectWorkflow(workflow: Workflow) {
  emit("select", workflow);
}

function runWorkflow(workflow: Workflow) {
  emit("run", workflow);
}

// Rename
function startRenameWorkflow(wf: Workflow) {
  renameTarget.value = { id: wf.id, type: "workflow" };
  renameValue.value = wf.name;
  showRenameDialog.value = true;
}

function startRenameFolder(folder: Folder) {
  renameTarget.value = { id: folder.id, type: "folder" };
  renameValue.value = folder.name;
  showRenameDialog.value = true;
}

async function confirmRename() {
  if (!renameValue.value.trim()) return;
  const target = renameTarget.value;
  try {
    if (!target) {
      // Creating a new folder
      const folder = await dbCreateFolder(renameValue.value);
      folders.value.push(folder);
      toast.add({
        severity: "success",
        summary: "已创建文件夹",
        detail: folder.name,
        life: 2000,
      });
    } else if (target.type === "workflow") {
      await dbUpdateWorkflow(target.id, renameValue.value);
      const wf = workflows.value.find(w => w.id === target.id);
      if (wf) {
        wf.name = renameValue.value;
        wf.updatedAt = new Date().toISOString();
      }
    } else {
      await dbRenameFolder(target.id, renameValue.value);
      const folder = folders.value.find(f => f.id === target.id);
      if (folder) folder.name = renameValue.value;
    }
    showRenameDialog.value = false;
    renameTarget.value = null;
  } catch (e: any) {
    toast.add({
      severity: "error",
      summary: "操作失败",
      detail: String(e),
      life: 4000,
    });
  }
}

// Delete
function confirmDeleteWorkflow(wf: Workflow) {
  deleteTargetId.value = wf.id;
  deleteTargetName.value = wf.name;
  deleteTargetType.value = "workflow";
  showDeleteDialog.value = true;
}

function confirmDeleteFolder(folder: Folder) {
  deleteTargetId.value = folder.id;
  deleteTargetName.value = folder.name;
  deleteTargetType.value = "folder";
  showDeleteDialog.value = true;
}

async function confirmDelete() {
  if (!deleteTargetId.value) return;
  try {
    if (deleteTargetType.value === "workflow") {
      await dbDeleteWorkflow(deleteTargetId.value);
      workflows.value = workflows.value.filter(
        w => w.id !== deleteTargetId.value,
      );
      emit("delete", deleteTargetId.value);
    } else {
      await dbDeleteFolder(deleteTargetId.value);
      folders.value = folders.value.filter(f => f.id !== deleteTargetId.value);
      workflows.value = workflows.value.filter(
        w => w.folderId !== deleteTargetId.value,
      );
    }
    showDeleteDialog.value = false;
  } catch (e: any) {
    toast.add({
      severity: "error",
      summary: "删除失败",
      detail: String(e),
      life: 4000,
    });
  }
}
</script>

<style scoped>
.workflow-manager {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.manager-header {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 12px;
  border-bottom: 1px solid var(--surface-border);
}

.manager-header :deep(.p-button) {
  flex-shrink: 0;
}

.search-box {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 12px;
  background: var(--surface-card);
  border: 1px solid var(--surface-border);
  border-radius: 6px;
}

.search-box i {
  color: var(--text-color-secondary);
  font-size: 0.875rem;
}

.search-box :deep(.p-inputtext) {
  flex: 1;
  background: transparent;
  border: none;
  padding: 0;
  font-size: 0.8125rem;
}

.folders-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.folder-group {
  margin-bottom: 6px;
}

.folder-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 10px;
  background: var(--surface-card);
  border: 1px solid var(--surface-border);
  border-radius: 8px;
  cursor: pointer;
  user-select: none;
  transition: background 0.15s;
}

.folder-header:hover {
  background: var(--surface-hover);
}

.folder-chevron {
  font-size: 0.75rem;
  color: var(--text-color-secondary);
}

.folder-icon {
  color: var(--primary-color);
  font-size: 0.875rem;
}

.folder-name {
  flex: 1;
  min-width: 0;
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--text-color);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.folder-count {
  font-size: 0.6875rem;
  color: var(--text-color-secondary);
  background: var(--surface-hover);
  padding: 1px 6px;
  border-radius: 10px;
  margin-right: 4px;
}

.folder-more {
  position: relative;
}

.popup-menu {
  position: absolute;
  top: 100%;
  right: 0;
  z-index: 100;
  min-width: 140px;
  background: var(--surface-card);
  border: 1px solid var(--surface-border);
  border-radius: 8px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.2);
  padding: 4px;
  margin-top: 4px;
}

.popup-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 8px 10px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--text-color);
  font-size: 0.8125rem;
  cursor: pointer;
  transition: background 0.1s;
}

.popup-item:hover {
  background: var(--surface-hover);
}

.popup-item i {
  font-size: 0.8125rem;
  width: 16px;
  text-align: center;
}

.popup-item-danger {
  color: var(--p-red-400);
}

.popup-item-danger:hover {
  background: rgba(239, 68, 68, 0.1);
}

/* Workflow rows */
.folder-workflows {
  padding: 4px 0 4px 12px;
}

.workflow-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 10px;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.1s;
}

.workflow-row:hover {
  background: var(--surface-hover);
}

.workflow-row.selected {
  background: rgba(99, 102, 241, 0.1);
  outline: 1px solid var(--primary-color);
}

.row-icon {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(99, 102, 241, 0.15);
  border-radius: 6px;
  color: var(--primary-color);
  font-size: 0.8125rem;
  flex-shrink: 0;
}

.row-content {
  flex: 1;
  min-width: 0;
}

.row-name {
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--text-color);
  display: block;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.row-date {
  font-size: 0.6875rem;
  color: var(--text-color-secondary);
  display: block;
  margin-top: 1px;
}

.row-actions {
  display: flex;
  gap: 2px;
  opacity: 0;
  transition: opacity 0.1s;
}

.workflow-row:hover .row-actions {
  opacity: 1;
}

.row-action-btn {
  width: 28px;
  height: 28px;
}

/* Empty states */
.empty-folder {
  padding: 12px;
  text-align: center;
  font-size: 0.75rem;
  color: var(--text-color-secondary);
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 32px;
  color: var(--text-color-secondary);
  gap: 12px;
}

.empty-state i {
  font-size: 2rem;
  opacity: 0.5;
}

/* Dialog */
.dialog-body {
  padding: 8px 0;
}

.form-field {
  margin-bottom: 12px;
}

.form-field label {
  display: block;
  font-size: 0.8125rem;
  font-weight: 500;
  margin-bottom: 4px;
  color: var(--text-color);
}

.delete-content {
  text-align: center;
}

.warning-icon {
  font-size: 48px;
  color: var(--yellow-500);
  margin-bottom: 12px;
}

.warning-text {
  color: var(--p-red-400);
  font-size: 0.8125rem;
}

.w-full {
  width: 100%;
}
</style>
