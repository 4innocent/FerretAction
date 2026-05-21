<script setup lang="ts">
import { computed } from 'vue'
import {
  Search, MousePointer, MousePointerClick, Keyboard, Clock, GitBranch, Repeat,
  Image as ImageIcon, Target, FolderOpen, Plus, Camera, ChevronDown, ChevronRight,
  ChevronUp, Zap, CheckCircle2, AlertCircle, XCircle, AlertTriangle, Info,
  Bug, Save, Play, Pause, Square, SkipForward, Settings, HelpCircle,
  FileDown, FileUp, History, Layers, Star, StarOff, MoreHorizontal,
  Copy, Trash2, Edit3, Terminal, Monitor, Crosshair, ZoomIn, ZoomOut,
  Maximize2, Lock, Unlock, RefreshCw, Download, GripVertical, Check, X,
  ImagePlus, FolderPlus, Loader2,
} from 'lucide-vue-next'

// ============================================================
// 动态图标组件
// 根据 icon name 字符串映射到对应的 lucide-vue-next 图标
// ============================================================

const iconMap: Record<string, any> = {
  search: Search, 'mouse-pointer': MousePointer, 'mouse-pointer-click': MousePointerClick,
  keyboard: Keyboard, clock: Clock, 'git-branch': GitBranch, repeat: Repeat,
  image: ImageIcon, target: Target, 'folder-open': FolderOpen, plus: Plus,
  camera: Camera, 'chevron-down': ChevronDown, 'chevron-right': ChevronRight,
  'chevron-up': ChevronUp, zap: Zap, 'check-circle-2': CheckCircle2,
  'alert-circle': AlertCircle, 'x-circle': XCircle, 'alert-triangle': AlertTriangle,
  info: Info, bug: Bug, save: Save, play: Play, pause: Pause, square: Square,
  'skip-forward': SkipForward, settings: Settings, 'help-circle': HelpCircle,
  'file-down': FileDown, 'file-up': FileUp, history: History, layers: Layers,
  star: Star, 'star-off': StarOff, 'more-horizontal': MoreHorizontal,
  copy: Copy, 'trash-2': Trash2, 'edit-3': Edit3, terminal: Terminal,
  monitor: Monitor, crosshair: Crosshair, 'zoom-in': ZoomIn, 'zoom-out': ZoomOut,
  'maximize-2': Maximize2, lock: Lock, unlock: Unlock, 'refresh-cw': RefreshCw,
  download: Download, 'grip-vertical': GripVertical, check: Check, x: X,
  'image-plus': ImagePlus, 'folder-plus': FolderPlus, 'loader-2': Loader2,
}

const props = defineProps<{
  name: string
  size?: number | string
  class?: string
}>()

const sizeValue = computed(() => props.size ?? 16)
</script>

<template>
  <component :is="iconMap[name]" v-if="iconMap[name]" :size="sizeValue" :class="$attrs.class" />
  <span v-else class="inline-block" :style="{ width: sizeValue + 'px', height: sizeValue + 'px' }" />
</template>
