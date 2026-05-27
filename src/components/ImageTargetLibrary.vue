<template>
  <div class="image-library">
    <div class="library-header">
      <div class="search-box" :style="{ maxWidth: Math.min(Math.max(100, sidebarWidth - 60), 280) + 'px' }">
        <i class="pi pi-search"></i>
        <InputText v-model="searchQuery" placeholder="搜索..." />
      </div>
      <Button
        icon="pi pi-folder-plus"
        size="small"
        @click="startAddFolder"
        v-tooltip="'新建文件夹'"
      />
    </div>

    <div class="folders-list">
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
            getFolderTargets(folder.id).length
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
                  startAddTarget(folder.id);
                  activeMenu = null;
                "
              >
                <i class="pi pi-plus"></i>
                <span>添加图片</span>
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
                  $emit('delete-folder', folder.id);
                  activeMenu = null;
                "
              >
                <i class="pi pi-trash"></i>
                <span>删除文件夹</span>
              </button>
            </div>
          </div>
        </div>

        <div v-if="expandedFolders.has(folder.id)" class="folder-targets">
          <div
            v-for="target in getFolderTargets(folder.id)"
            :key="target.id"
            class="target-row"
            :class="{ selected: selectedId === target.id }"
            @click="selectTarget(target)"
            draggable="true"
            @dragstart="onDragStart($event, target)"
          >
            <div class="row-thumbnail">
              <div
                v-if="target.thumbnail"
                class="thumbnail-image"
                :style="{ backgroundImage: `url(${target.thumbnail})` }"
              ></div>
              <div v-else class="thumbnail-placeholder">
                <i class="pi pi-image"></i>
              </div>
            </div>
            <span class="row-name">{{ target.name }}</span>
            <div class="row-actions">
              <Button
                icon="pi pi-pencil"
                text
                size="small"
                class="row-action-btn"
                @click.stop="startRenameTarget(target)"
                v-tooltip="'重命名'"
              />
              <Button
                icon="pi pi-trash"
                text
                size="small"
                severity="danger"
                class="row-action-btn"
                @click.stop="$emit('delete', target.id)"
                v-tooltip="'删除'"
              />
            </div>
          </div>

          <div
            v-if="getFolderTargets(folder.id).length === 0"
            class="empty-folder"
          >
            <span>文件夹为空</span>
          </div>
        </div>
      </div>

      <div v-if="folders.length === 0" class="empty-state">
        <i class="pi pi-folder-open"></i>
        <p>暂无文件夹</p>
        <Button
          label="新建文件夹"
          icon="pi pi-plus"
          size="small"
          @click="startAddFolder"
        />
      </div>
    </div>

    <!-- Hidden file input -->
    <input
      ref="fileInputRef"
      type="file"
      accept="image/*"
      style="display: none"
      @change="onFileSelected"
    />

    <!-- Add Folder Dialog -->
    <Dialog
      v-model:visible="showAddFolderDialog"
      header="新建文件夹"
      :modal="true"
      :style="{ width: '360px' }"
    >
      <div class="dialog-body">
        <InputText
          v-model="newFolderName"
          placeholder="文件夹名称"
          class="w-full"
          @keyup.enter="confirmAddFolder"
        />
      </div>
      <template #footer>
        <Button label="取消" text @click="showAddFolderDialog = false" />
        <Button label="确定" @click="confirmAddFolder" />
      </template>
    </Dialog>

    <!-- Rename Folder Dialog -->
    <Dialog
      v-model:visible="showRenameDialog"
      header="重命名文件夹"
      :modal="true"
      :style="{ width: '360px' }"
    >
      <div class="dialog-body">
        <InputText
          v-model="renameValue"
          placeholder="文件夹名称"
          class="w-full"
          @keyup.enter="confirmRename"
        />
      </div>
      <template #footer>
        <Button label="取消" text @click="showRenameDialog = false" />
        <Button label="确定" @click="confirmRename" />
      </template>
    </Dialog>

    <!-- Import Target Name Dialog -->
    <Dialog
      v-model:visible="showImportNameDialog"
      header="导入图片"
      :modal="true"
      :style="{ width: '400px' }"
    >
      <div class="dialog-body">
        <div class="import-preview" v-if="importThumbnail">
          <div
            class="preview-image"
            :style="{ backgroundImage: `url(${importThumbnail})` }"
          ></div>
        </div>
        <InputText
          v-model="importNameValue"
          placeholder="图片名称"
          class="w-full"
          @keyup.enter="confirmImportTarget"
        />
      </div>
      <template #footer>
        <Button label="取消" text @click="showImportNameDialog = false" />
        <Button label="导入" @click="confirmImportTarget" />
      </template>
    </Dialog>

    <!-- Rename Target Dialog -->
    <Dialog
      v-model:visible="showRenameTargetDialog"
      header="重命名图片"
      :modal="true"
      :style="{ width: '360px' }"
    >
      <div class="dialog-body">
        <InputText
          v-model="renameTargetValue"
          placeholder="图片名称"
          class="w-full"
          @keyup.enter="confirmRenameTarget"
        />
      </div>
      <template #footer>
        <Button label="取消" text @click="showRenameTargetDialog = false" />
        <Button label="确定" @click="confirmRenameTarget" />
      </template>
    </Dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import Button from "primevue/button";
import InputText from "primevue/inputtext";
import Dialog from "primevue/dialog";
import type { ImageTarget, ImageFolder } from "../types";

const props = defineProps<{
  targets: ImageTarget[];
  folders: ImageFolder[];
  sidebarWidth: number;
}>();

const emit = defineEmits<{
  select: [target: ImageTarget];
  add: [data: { name: string; thumbnail: string; folderId: string }];
  delete: [id: string];
  "add-folder": [name: string];
  "delete-folder": [id: string];
  "rename-folder": [id: string, name: string];
  "rename-target": [id: string, name: string];
}>();

const searchQuery = ref("");
const selectedId = ref<string | null>(null);
const expandedFolders = ref<Set<string>>(new Set());
const fileInputRef = ref<HTMLInputElement | null>(null);
const pendingFolderId = ref<string | null>(null);
const activeMenu = ref<string | null>(null);

const showAddFolderDialog = ref(false);
const newFolderName = ref("");
const showRenameDialog = ref(false);
const renameValue = ref("");
const renamingFolderId = ref<string | null>(null);

// Target rename
const showRenameTargetDialog = ref(false);
const renameTargetValue = ref("");
const renamingTargetId = ref<string | null>(null);

// Target name on import
const showImportNameDialog = ref(false);
const importNameValue = ref("");
const importThumbnail = ref("");
const importFolderId = ref<string | null>(null);

// Close popup menu on outside click
const closeMenu = (e: MouseEvent) => {
  const target = e.target as HTMLElement;
  if (!target.closest(".folder-more")) {
    activeMenu.value = null;
  }
};
onMounted(() => document.addEventListener("click", closeMenu));
onUnmounted(() => document.removeEventListener("click", closeMenu));

const filteredFolders = computed(() => {
  if (!searchQuery.value) return props.folders;
  const query = searchQuery.value.toLowerCase();
  return props.folders.filter(f => {
    if (f.name.toLowerCase().includes(query)) return true;
    return getFolderTargets(f.id).some(t =>
      t.name.toLowerCase().includes(query),
    );
  });
});

const getFolderTargets = (folderId: string) =>
  props.targets.filter(t => t.folderId === folderId);

const toggleFolder = (folderId: string) => {
  if (expandedFolders.value.has(folderId)) {
    expandedFolders.value.delete(folderId);
  } else {
    expandedFolders.value.add(folderId);
  }
};

const toggleMenu = (folderId: string) => {
  activeMenu.value = activeMenu.value === folderId ? null : folderId;
};

const selectTarget = (target: ImageTarget) => {
  selectedId.value = target.id;
  emit("select", target);
};

const startAddTarget = (folderId: string) => {
  pendingFolderId.value = folderId;
  fileInputRef.value?.click();
};

const onFileSelected = (event: Event) => {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  if (!file || !pendingFolderId.value) return;

  const reader = new FileReader();
  reader.onload = () => {
    importNameValue.value = file.name.replace(/\.[^.]+$/, "");
    importThumbnail.value = reader.result as string;
    importFolderId.value = pendingFolderId.value;
    showImportNameDialog.value = true;
    pendingFolderId.value = null;
  };
  reader.readAsDataURL(file);
  input.value = "";
};

const confirmImportTarget = () => {
  const name = importNameValue.value.trim();
  if (!name || !importFolderId.value) return;
  emit("add", {
    name,
    thumbnail: importThumbnail.value,
    folderId: importFolderId.value,
  });
  showImportNameDialog.value = false;
};

const startRenameTarget = (target: ImageTarget) => {
  renamingTargetId.value = target.id;
  renameTargetValue.value = target.name;
  showRenameTargetDialog.value = true;
};

const confirmRenameTarget = () => {
  const name = renameTargetValue.value.trim();
  if (!name || !renamingTargetId.value) return;
  emit("rename-target", renamingTargetId.value, name);
  showRenameTargetDialog.value = false;
};

const startAddFolder = () => {
  newFolderName.value = "";
  showAddFolderDialog.value = true;
};

const confirmAddFolder = () => {
  const name = newFolderName.value.trim();
  if (!name) return;
  emit("add-folder", name);
  showAddFolderDialog.value = false;
};

const startRenameFolder = (folder: ImageFolder) => {
  renamingFolderId.value = folder.id;
  renameValue.value = folder.name;
  showRenameDialog.value = true;
};

const confirmRename = () => {
  const name = renameValue.value.trim();
  if (!name || !renamingFolderId.value) return;
  emit("rename-folder", renamingFolderId.value, name);
  showRenameDialog.value = false;
};

const onDragStart = (event: DragEvent, target: ImageTarget) => {
  event.dataTransfer?.setData(
    "application/json",
    JSON.stringify({
      type: "image-target",
      targetId: target.id,
      targetName: target.name,
    }),
  );
  event.dataTransfer!.effectAllowed = "copy";
};
</script>

<style scoped>
.image-library {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.library-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px;
  border-bottom: 1px solid var(--surface-border);
}

.library-header :deep(.p-button) {
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

/* Target list rows */
.folder-targets {
  padding: 4px 0 4px 12px;
}

.target-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 6px 8px;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.1s;
}

.target-row:hover {
  background: var(--surface-hover);
}

.target-row.selected {
  background: rgba(99, 102, 241, 0.1);
  outline: 1px solid var(--primary-color);
}

.row-thumbnail {
  width: 32px;
  height: 32px;
  border-radius: 4px;
  overflow: hidden;
  flex-shrink: 0;
}

.row-thumbnail .thumbnail-image {
  width: 100%;
  height: 100%;
  background-size: cover;
  background-position: center;
}

.row-thumbnail .thumbnail-placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--surface-hover);
  color: var(--text-color-secondary);
  font-size: 0.75rem;
}

.row-name {
  flex: 1;
  font-size: 0.8125rem;
  color: var(--text-color);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.row-actions {
  display: flex;
  gap: 2px;
  opacity: 0;
  transition: opacity 0.1s;
}

.target-row:hover .row-actions {
  opacity: 1;
}

.row-action-btn {
  width: 28px;
  height: 28px;
}

/* Import preview */
.import-preview {
  display: flex;
  justify-content: center;
  margin-bottom: 12px;
}

.preview-image {
  width: 120px;
  height: 120px;
  border-radius: 8px;
  border: 1px solid var(--surface-border);
  background-size: contain;
  background-repeat: no-repeat;
  background-position: center;
}

.empty-folder {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px;
  font-size: 0.75rem;
  color: var(--text-color-secondary);
}

.add-link {
  display: flex;
  align-items: center;
  gap: 4px;
  border: none;
  background: transparent;
  color: var(--primary-color);
  font-size: 0.75rem;
  cursor: pointer;
  padding: 2px 4px;
  border-radius: 4px;
}

.add-link:hover {
  background: rgba(99, 102, 241, 0.1);
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

.dialog-body {
  padding: 8px 0;
}

.w-full {
  width: 100%;
}
</style>
