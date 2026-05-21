<script setup lang="ts">
import type { MatchResult, ScreenRegion } from '../../types'
import Button from '../ui/Button.vue'
import Badge from '../ui/Badge.vue'
import DynamicIcon from '../DynamicIcon.vue'

// ============================================================
// 屏幕调试面板
// 显示截图预览、匹配结果高亮框和匹配列表
// ============================================================

defineProps<{
  screenshot: string | null
  matchResults: MatchResult[]
  searchRegion: ScreenRegion | null
  isSearching: boolean
  selectedMatchId: string | null
}>()

const emit = defineEmits<{
  capture: []
  search: []
  selectMatch: [id: string | null]
}>()

/** 匹配区域框的边框样式 */
function matchBorderClass(match: MatchResult, isSelected: boolean) {
  if (isSelected) return 'border-primary bg-primary/20'
  if (match.confidence >= 0.9) return 'border-[oklch(0.72_0.18_165)] bg-[oklch(0.72_0.18_165/0.1)]'
  if (match.confidence >= 0.7) return 'border-[oklch(0.7_0.18_85)] bg-[oklch(0.7_0.18_85/0.1)]'
  return 'border-destructive bg-destructive/10'
}

function matchBadgeClass(match: MatchResult) {
  if (match.confidence >= 0.9) return 'bg-[oklch(0.72_0.18_165)] text-[oklch(0.13_0.005_260)]'
  if (match.confidence >= 0.7) return 'bg-[oklch(0.7_0.18_85)] text-[oklch(0.13_0.005_260)]'
  return 'bg-destructive text-destructive-foreground'
}
</script>

<template>
  <div class="flex h-full flex-col">
    <!-- 工具栏 -->
    <div class="flex items-center justify-between border-b border-border px-3 py-2">
      <div class="flex items-center gap-2">
        <span class="text-sm font-medium">屏幕识别</span>
        <Badge v-if="isSearching" variant="secondary" class="text-[10px] animate-pulse">
          搜索中...
        </Badge>
      </div>
      <div class="flex items-center gap-1">
        <Button variant="ghost" size="sm" class="h-7 px-2" @click="emit('capture')">
          <DynamicIcon name="camera" :size="12" class="mr-1" />
          截图
        </Button>
        <Button variant="ghost" size="sm" class="h-7 px-2" @click="emit('search')">
          <DynamicIcon name="target" :size="12" class="mr-1" />
          搜索
        </Button>
      </div>
    </div>

    <!-- 截图预览区 -->
    <div class="flex-1 relative overflow-hidden bg-[oklch(0.08_0.005_260)]">
      <div v-if="screenshot" class="relative h-full w-full">
        <!-- 截图图片 -->
        <img :src="screenshot" alt="Screen capture" class="h-full w-full object-contain" />

        <!-- 搜索区域叠加 -->
        <div
          v-if="searchRegion"
          class="absolute border-2 border-dashed border-primary/50 bg-primary/10"
          :style="{
            left: `${(searchRegion.x / 1920) * 100}%`,
            top: `${(searchRegion.y / 1080) * 100}%`,
            width: `${(searchRegion.width / 1920) * 100}%`,
            height: `${(searchRegion.height / 1080) * 100}%`,
          }"
        />

        <!-- 匹配结果叠加框 -->
        <div
          v-for="match in matchResults"
          :key="match.id"
          class="absolute border-2 cursor-pointer transition-all"
          :class="matchBorderClass(match, match.id === selectedMatchId)"
          :style="{
            left: `${(match.region.x / 1920) * 100}%`,
            top: `${(match.region.y / 1080) * 100}%`,
            width: `${(match.region.width / 1920) * 100}%`,
            height: `${(match.region.height / 1080) * 100}%`,
          }"
          @click="emit('selectMatch', match.id)"
        >
          <!-- 匹配置信度标签 -->
          <div class="absolute -top-5 left-0 rounded px-1.5 py-0.5 text-[10px] font-mono" :class="matchBadgeClass(match)">
            {{ Math.round(match.confidence * 100) }}%
          </div>
          <!-- 中心十字准星 -->
          <DynamicIcon name="crosshair" :size="16" class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 text-foreground/70" />
        </div>

        <!-- 缩放控件 -->
        <div class="absolute bottom-3 right-3 flex items-center gap-1 rounded-lg border border-border bg-card/95 p-1 backdrop-blur">
          <Button variant="ghost" size="icon" class="h-7 w-7">
            <DynamicIcon name="zoom-out" :size="14" />
          </Button>
          <span class="w-10 text-center text-xs text-muted-foreground">100%</span>
          <Button variant="ghost" size="icon" class="h-7 w-7">
            <DynamicIcon name="zoom-in" :size="14" />
          </Button>
        </div>
      </div>

      <!-- 空状态 -->
      <div v-else class="flex h-full items-center justify-center">
        <div class="text-center">
          <DynamicIcon name="camera" :size="48" class="mx-auto text-muted-foreground/30" />
          <p class="mt-2 text-sm text-muted-foreground">点击截图开始</p>
        </div>
      </div>
    </div>

    <!-- 匹配结果列表 -->
    <div v-if="matchResults.length > 0" class="border-t border-border p-3">
      <div class="flex items-center justify-between mb-2">
        <span class="text-xs text-muted-foreground">找到 {{ matchResults.length }} 个匹配</span>
        <Button variant="ghost" size="sm" class="h-6 px-2 text-xs" @click="emit('search')">
          <DynamicIcon name="refresh-cw" :size="12" class="mr-1" />
          刷新
        </Button>
      </div>
      <div class="flex flex-wrap gap-2">
        <Button
          v-for="(match, index) in matchResults"
          :key="match.id"
          :variant="selectedMatchId === match.id ? 'default' : 'outline'"
          size="sm"
          class="h-7 px-2 text-xs"
          @click="emit('selectMatch', match.id)"
        >
          <DynamicIcon
            v-if="match.confidence >= 0.9"
            name="check"
            :size="12"
            class="mr-1 text-[oklch(0.72_0.18_165)]"
          />
          <DynamicIcon
            v-else-if="match.confidence < 0.7"
            name="x"
            :size="12"
            class="mr-1 text-destructive"
          />
          #{{ index + 1 }} ({{ Math.round(match.confidence * 100) }}%)
        </Button>
      </div>
    </div>
  </div>
</template>
