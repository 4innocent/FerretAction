<script setup lang="ts">
import { computed } from 'vue'
import type { ImageTargetData } from '../../types'
import DynamicIcon from '../DynamicIcon.vue'
import Button from '../ui/Button.vue'
import Badge from '../ui/Badge.vue'
import { DropdownMenu, DropdownMenuTrigger, DropdownMenuContent, DropdownMenuItem } from '../ui'

// ============================================================
// 图片目标项 / 图片目标列表
// 用于左侧边栏，展示和管理屏幕截图匹配目标
// ============================================================

// ---- 单个图片目标 ----
const targetProps = defineProps<{
  target: ImageTargetData
  isSelected: boolean
}>()

const targetEmit = defineEmits<{
  select: []
  delete: []
  duplicate: []
}>()

const confidenceClass = computed(() => {
  if (targetProps.target.confidence >= 0.9) return 'bg-[oklch(0.72_0.18_165/0.2)] text-[oklch(0.72_0.18_165)]'
  if (targetProps.target.confidence >= 0.7) return 'bg-[oklch(0.7_0.18_85/0.2)] text-[oklch(0.7_0.18_85)]'
  return 'bg-destructive/20 text-destructive'
})
</script>

<template>
  <!-- 单张图片目标卡片 -->
  <div
    class="group relative rounded-lg border bg-card p-2 transition-all cursor-pointer"
    :class="isSelected ? 'border-primary ring-1 ring-primary/30' : 'border-border hover:border-muted-foreground/50'"
    @click="targetEmit('select')"
  >
    <div class="flex items-start gap-2">
      <!-- 缩略图占位 -->
      <div class="relative h-12 w-12 flex-shrink-0 rounded border border-border bg-secondary overflow-hidden">
        <img v-if="target.thumbnail" :src="target.thumbnail" :alt="target.name" class="h-full w-full object-cover" />
        <div v-else class="flex h-full w-full items-center justify-center">
          <DynamicIcon name="image" :size="20" class="text-muted-foreground" />
        </div>
        <div class="absolute bottom-0 right-0 rounded-tl bg-background/80 px-1">
          <DynamicIcon name="target" :size="12" class="text-primary" />
        </div>
      </div>

      <!-- 目标信息 -->
      <div class="flex-1 min-w-0">
        <div class="flex items-center justify-between">
          <span class="text-sm font-medium truncate">{{ target.name }}</span>
          <DropdownMenu>
            <DropdownMenuTrigger>
              <Button variant="ghost" size="icon" class="h-6 w-6 opacity-0 group-hover:opacity-100">
                <DynamicIcon name="more-horizontal" :size="14" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              <DropdownMenuItem @click="targetEmit('duplicate')">
                <DynamicIcon name="copy" :size="14" class="mr-2" />
                复制
              </DropdownMenuItem>
              <DropdownMenuItem @click="targetEmit('delete')" class="text-destructive">
                <DynamicIcon name="trash-2" :size="14" class="mr-2" />
                删除
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
        <div class="mt-1 flex items-center gap-2">
          <Badge variant="secondary" :class="confidenceClass">
            {{ Math.round(target.confidence * 100) }}%
          </Badge>
          <span class="text-[10px] text-muted-foreground">使用 {{ target.matchCount }} 次</span>
        </div>
      </div>
    </div>
  </div>
</template>
