<template>
  <div class="image-library">
    <div class="library-header">
      <div class="search-box">
        <i class="pi pi-search"></i>
        <InputText v-model="searchQuery" placeholder="搜索图像目标..." />
      </div>
      <Button icon="pi pi-plus" size="small" @click="$emit('add')" v-tooltip="'添加新目标'" />
    </div>

    <div class="targets-list">
      <div 
        v-for="target in filteredTargets" 
        :key="target.id"
        class="target-item"
        :class="{ selected: selectedId === target.id }"
        @click="selectTarget(target)"
        draggable="true"
        @dragstart="onDragStart($event, target)"
      >
        <div class="target-thumbnail">
          <div v-if="target.thumbnail" class="thumbnail-image" :style="{ backgroundImage: `url(${target.thumbnail})` }"></div>
          <div v-else class="thumbnail-placeholder">
            <i class="pi pi-image"></i>
          </div>
        </div>
        
        <div class="target-info">
          <div class="target-name">{{ target.name }}</div>
          <div class="target-meta">
            <span class="threshold">
              <i class="pi pi-percentage"></i>
              {{ Math.round(target.matchThreshold * 100) }}%
            </span>
            <span v-if="target.lastMatch" class="last-match status-success">
              <i class="pi pi-check-circle"></i>
              {{ target.lastMatch.confidence.toFixed(2) }}
            </span>
            <span v-else class="last-match status-warning">
              <i class="pi pi-exclamation-circle"></i>
              未匹配
            </span>
          </div>
        </div>
        
        <div class="target-actions">
          <Button 
            icon="pi pi-pencil" 
            text 
            size="small" 
            @click.stop="editTarget(target)"
          />
          <Button 
            icon="pi pi-trash" 
            text 
            size="small" 
            severity="danger"
            @click.stop="$emit('delete', target.id)"
          />
        </div>
      </div>
      
      <div v-if="filteredTargets.length === 0" class="empty-state">
        <i class="pi pi-inbox"></i>
        <p>未找到图像目标</p>
        <Button label="添加目标" icon="pi pi-plus" size="small" @click="$emit('add')" />
      </div>
    </div>

    <div class="library-footer">
      <div class="stats">
        <span>{{ targets.length }} 个目标</span>
        <span>{{ matchedCount }} 已匹配</span>
      </div>
      <Button label="批量导入" icon="pi pi-upload" text size="small" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import Button from 'primevue/button'
import InputText from 'primevue/inputtext'
import type { ImageTarget } from '../types'

const props = defineProps<{
  targets: ImageTarget[]
}>()

const emit = defineEmits<{
  select: [target: ImageTarget]
  add: []
  delete: [id: string]
}>()

const searchQuery = ref('')
const selectedId = ref<string | null>(null)

const filteredTargets = computed(() => {
  if (!searchQuery.value) return props.targets
  const query = searchQuery.value.toLowerCase()
  return props.targets.filter(t => t.name.toLowerCase().includes(query))
})

const matchedCount = computed(() => 
  props.targets.filter(t => t.lastMatch !== null).length
)

const selectTarget = (target: ImageTarget) => {
  selectedId.value = target.id
  emit('select', target)
}

const editTarget = (target: ImageTarget) => {
  // Open edit dialog
}

const onDragStart = (event: DragEvent, target: ImageTarget) => {
  event.dataTransfer?.setData('application/json', JSON.stringify({
    type: 'image-target',
    targetId: target.id,
    targetName: target.name
  }))
}
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

.search-box {
  flex: 1;
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

.targets-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.target-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px;
  margin-bottom: 6px;
  background: var(--surface-card);
  border: 1px solid var(--surface-border);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.target-item:hover {
  border-color: var(--primary-color);
  background: var(--surface-hover);
}

.target-item.selected {
  border-color: var(--primary-color);
  background: rgba(99, 102, 241, 0.1);
}

.target-thumbnail {
  width: 48px;
  height: 48px;
  border-radius: 6px;
  overflow: hidden;
  flex-shrink: 0;
}

.thumbnail-image {
  width: 100%;
  height: 100%;
  background-size: cover;
  background-position: center;
}

.thumbnail-placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--surface-hover);
  color: var(--text-color-secondary);
}

.target-info {
  flex: 1;
  min-width: 0;
}

.target-name {
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--text-color);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.target-meta {
  display: flex;
  gap: 12px;
  margin-top: 4px;
  font-size: 0.6875rem;
}

.threshold {
  display: flex;
  align-items: center;
  gap: 4px;
  color: var(--text-color-secondary);
}

.last-match {
  display: flex;
  align-items: center;
  gap: 4px;
}

.target-actions {
  display: flex;
  gap: 2px;
  opacity: 0;
  transition: opacity 0.2s;
}

.target-item:hover .target-actions {
  opacity: 1;
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

.library-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  border-top: 1px solid var(--surface-border);
  font-size: 0.6875rem;
  color: var(--text-color-secondary);
}

.stats {
  display: flex;
  gap: 16px;
}
</style>
