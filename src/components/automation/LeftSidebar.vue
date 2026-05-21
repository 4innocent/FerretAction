<script setup lang="ts">
import { ref, computed } from 'vue'
import type { ImageTargetData, NodeType } from '../../types'
import { NODE_BLOCKS } from '../../types'
import Button from '../ui/Button.vue'
import Input from '../ui/Input.vue'
import Separator from '../ui/Separator.vue'
import { Collapsible, CollapsibleTrigger, CollapsibleContent } from '../ui'
import ImageTargetList from './ImageTargetList.vue'
import NodeBlock from './NodeBlock.vue'
import DynamicIcon from '../DynamicIcon.vue'

// ============================================================
// 左侧边栏
// 包含搜索框、图片目标列表（可折叠）和可拖拽的操作节点积木
// ============================================================

defineProps<{
  imageTargets: ImageTargetData[]
  selectedTargetId: string | null
}>()

const emit = defineEmits<{
  selectTarget: [id: string]
  addTarget: []
  deleteTarget: [id: string]
  duplicateTarget: [id: string]
  dragNodeStart: [e: DragEvent, type: NodeType]
}>()

const searchQuery = ref('')

// 根据搜索词过滤
// 搜索过滤在父组件中处理，这里仅用于节点积木块过滤

const filteredBlocks = computed(() =>
  NODE_BLOCKS.filter(
    (b) =>
      b.label.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
      b.description.toLowerCase().includes(searchQuery.value.toLowerCase())
  )
)
</script>

<template>
  <div class="flex h-full flex-col border-r border-border bg-sidebar">
    <!-- 搜索框 -->
    <div class="p-3 border-b border-border">
      <div class="relative">
        <DynamicIcon name="search" :size="16" class="absolute left-2.5 top-1/2 -translate-y-1/2 text-muted-foreground" />
        <Input v-model="searchQuery" placeholder="搜索..." class="h-8 pl-9 text-xs bg-secondary border-0" />
      </div>
    </div>

    <div class="flex-1 overflow-auto">
      <!-- 图片目标区块（可折叠） -->
      <Collapsible :defaultOpen="true">
        <CollapsibleTrigger class="flex w-full items-center justify-between px-3 py-2 hover:bg-secondary/50 transition-colors">
          <div class="flex items-center gap-2">
            <DynamicIcon name="image" :size="16" class="text-muted-foreground" />
            <span class="text-xs font-medium uppercase tracking-wider text-muted-foreground">图片目标</span>
          </div>
          <div class="flex items-center gap-1">
            <Button variant="ghost" size="icon" class="h-6 w-6" @click.stop="emit('addTarget')">
              <DynamicIcon name="plus" :size="14" />
            </Button>
            <DynamicIcon name="chevron-down" :size="16" class="text-muted-foreground" />
          </div>
        </CollapsibleTrigger>
        <CollapsibleContent>
          <div class="px-3 pb-3">
            <ImageTargetList
              v-if="imageTargets.length > 0"
              :targets="imageTargets"
              :selectedId="selectedTargetId"
              @select="emit('selectTarget', $event)"
              @delete="emit('deleteTarget', $event)"
              @duplicate="emit('duplicateTarget', $event)"
            />
            <!-- 空状态 -->
            <div v-else class="py-6 text-center">
              <DynamicIcon name="image-plus" :size="32" class="mx-auto text-muted-foreground/30" />
              <p class="mt-2 text-xs text-muted-foreground">暂无图片目标</p>
              <Button variant="outline" size="sm" class="mt-2 h-7 text-xs" @click="emit('addTarget')">
                <DynamicIcon name="plus" :size="12" class="mr-1" />
                添加目标
              </Button>
            </div>
          </div>
        </CollapsibleContent>
      </Collapsible>

      <Separator />

      <!-- 操作节点区块（可折叠） -->
      <Collapsible :defaultOpen="true">
        <CollapsibleTrigger class="flex w-full items-center justify-between px-3 py-2 hover:bg-secondary/50 transition-colors">
          <div class="flex items-center gap-2">
            <DynamicIcon name="folder-open" :size="16" class="text-muted-foreground" />
            <span class="text-xs font-medium uppercase tracking-wider text-muted-foreground">操作节点</span>
          </div>
          <DynamicIcon name="chevron-down" :size="16" class="text-muted-foreground" />
        </CollapsibleTrigger>
        <CollapsibleContent>
          <div class="px-3 pb-3 space-y-2">
            <NodeBlock
              v-for="block in filteredBlocks"
              :key="block.type"
              :type="block.type"
              :label="block.label"
              :description="block.description"
              @dragStart="emit('dragNodeStart', $event, block.type)"
            />
          </div>
        </CollapsibleContent>
      </Collapsible>
    </div>

    <!-- 底部快捷操作 -->
    <div class="border-t border-border p-3">
      <Button variant="outline" size="sm" class="w-full h-8 text-xs">
        <DynamicIcon name="folder-open" :size="14" class="mr-2" />
        导入工作流
      </Button>
    </div>
  </div>
</template>
