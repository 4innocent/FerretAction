<template>
  <div class="screen-debugger">
    <div class="debugger-toolbar">
      <Button icon="pi pi-camera" label="捕获屏幕" @click="$emit('capture')" />
      <Button icon="pi pi-search" label="测试识别" severity="secondary" @click="$emit('test-recognition')" />
      <Button icon="pi pi-refresh" label="刷新" severity="secondary" outlined />
    </div>
    
    <div class="debugger-content">
      <div class="screen-preview">
        <div class="preview-container" ref="previewContainer">
          <!-- Placeholder screenshot area -->
          <div class="screenshot-placeholder">
            <div class="placeholder-content">
              <i class="pi pi-desktop"></i>
              <span>屏幕预览区域</span>
              <span class="hint">点击「捕获屏幕」获取当前屏幕截图</span>
            </div>
            
            <!-- Simulated detected regions -->
            <div 
              v-for="region in detectedRegions" 
              :key="region.id"
              class="detected-region"
              :style="getRegionStyle(region)"
              @click="selectRegion(region)"
            >
              <div class="region-label">
                {{ region.targetName }}
                <span class="confidence">{{ (region.confidence * 100).toFixed(0) }}%</span>
              </div>
            </div>
            
            <!-- Crosshair cursor indicator -->
            <div class="cursor-crosshair" :style="cursorStyle" v-show="showCursor">
              <div class="crosshair-h"></div>
              <div class="crosshair-v"></div>
              <div class="cursor-coords">{{ cursorX }}, {{ cursorY }}</div>
            </div>
          </div>
        </div>
        
        <div class="preview-info">
          <span>分辨率: 1920 × 1080</span>
          <span>缩放: {{ Math.round(previewScale * 100) }}%</span>
        </div>
      </div>
      
      <div class="detection-results">
        <div class="results-header">
          <span class="title">识别结果</span>
          <span class="count">{{ detectedRegions.length }} 个匹配</span>
        </div>
        
        <div class="results-list">
          <div 
            v-for="region in detectedRegions" 
            :key="region.id"
            class="result-item"
            :class="{ selected: selectedRegion?.id === region.id }"
            @click="selectRegion(region)"
          >
            <div class="result-icon" :style="{ background: getConfidenceColor(region.confidence) }">
              <i class="pi pi-check"></i>
            </div>
            <div class="result-info">
              <div class="result-name">{{ region.targetName }}</div>
              <div class="result-meta">
                <span>位置: ({{ region.x }}, {{ region.y }})</span>
                <span>尺寸: {{ region.width }} × {{ region.height }}</span>
              </div>
            </div>
            <div class="result-confidence">
              <span class="confidence-value">{{ (region.confidence * 100).toFixed(1) }}%</span>
              <div class="confidence-bar">
                <div class="confidence-fill" :style="{ width: `${region.confidence * 100}%`, background: getConfidenceColor(region.confidence) }"></div>
              </div>
            </div>
          </div>
          
          <div v-if="detectedRegions.length === 0" class="no-results">
            <i class="pi pi-search"></i>
            <span>暂无识别结果</span>
          </div>
        </div>
        
        <div class="selected-region-details" v-if="selectedRegion">
          <div class="details-header">选中区域详情</div>
          <div class="details-grid">
            <div class="detail-item">
              <label>目标名称</label>
              <span>{{ selectedRegion.targetName }}</span>
            </div>
            <div class="detail-item">
              <label>中心坐标</label>
              <span>({{ selectedRegion.x + selectedRegion.width / 2 }}, {{ selectedRegion.y + selectedRegion.height / 2 }})</span>
            </div>
            <div class="detail-item">
              <label>边界框</label>
              <span>{{ selectedRegion.x }}, {{ selectedRegion.y }}, {{ selectedRegion.width }}, {{ selectedRegion.height }}</span>
            </div>
            <div class="detail-item">
              <label>置信度</label>
              <span :style="{ color: getConfidenceColor(selectedRegion.confidence) }">{{ (selectedRegion.confidence * 100).toFixed(2) }}%</span>
            </div>
          </div>
          <div class="details-actions">
            <Button label="复制坐标" icon="pi pi-copy" size="small" outlined />
            <Button label="添加到工作流" icon="pi pi-plus" size="small" />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import Button from 'primevue/button'
import type { DetectedRegion } from '../types'

const props = defineProps<{
  currentScreenshot: string
  detectedRegions: DetectedRegion[]
}>()

const emit = defineEmits<{
  capture: []
  'test-recognition': []
}>()

const previewContainer = ref<HTMLElement>()
const previewScale = ref(0.5)
const selectedRegion = ref<DetectedRegion | null>(null)
const showCursor = ref(false)
const cursorX = ref(0)
const cursorY = ref(0)

const cursorStyle = computed(() => ({
  left: `${cursorX.value * previewScale.value}px`,
  top: `${cursorY.value * previewScale.value}px`
}))

const getRegionStyle = (region: DetectedRegion) => ({
  left: `${(region.x / 1920) * 100}%`,
  top: `${(region.y / 1080) * 100}%`,
  width: `${(region.width / 1920) * 100}%`,
  height: `${(region.height / 1080) * 100}%`,
  borderColor: getConfidenceColor(region.confidence)
})

const getConfidenceColor = (confidence: number): string => {
  if (confidence >= 0.9) return 'var(--green-400)'
  if (confidence >= 0.8) return 'var(--amber-400)'
  return 'var(--red-400)'
}

const selectRegion = (region: DetectedRegion) => {
  selectedRegion.value = region
}
</script>

<style scoped>
.screen-debugger {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.debugger-toolbar {
  display: flex;
  gap: 8px;
  padding: 8px 16px;
  border-bottom: 1px solid var(--surface-border);
}

.debugger-content {
  display: flex;
  flex: 1;
  overflow: hidden;
}

.screen-preview {
  flex: 1;
  display: flex;
  flex-direction: column;
  padding: 16px;
  border-right: 1px solid var(--surface-border);
}

.preview-container {
  flex: 1;
  position: relative;
  background: #000;
  border-radius: 8px;
  overflow: hidden;
}

.screenshot-placeholder {
  width: 100%;
  height: 100%;
  position: relative;
  background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
}

.placeholder-content {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  color: var(--text-color-secondary);
}

.placeholder-content i {
  font-size: 2rem;
  opacity: 0.5;
}

.placeholder-content .hint {
  font-size: 0.75rem;
  opacity: 0.7;
}

.detected-region {
  position: absolute;
  border: 2px solid;
  background: rgba(99, 102, 241, 0.1);
  cursor: pointer;
  transition: all 0.2s;
}

.detected-region:hover {
  background: rgba(99, 102, 241, 0.2);
}

.region-label {
  position: absolute;
  top: -24px;
  left: 0;
  padding: 2px 8px;
  background: var(--surface-overlay);
  border-radius: 4px;
  font-size: 0.6875rem;
  white-space: nowrap;
}

.region-label .confidence {
  margin-left: 6px;
  color: var(--green-400);
}

.cursor-crosshair {
  position: absolute;
  pointer-events: none;
}

.crosshair-h,
.crosshair-v {
  position: absolute;
  background: var(--red-400);
}

.crosshair-h {
  width: 20px;
  height: 1px;
  left: -10px;
}

.crosshair-v {
  width: 1px;
  height: 20px;
  top: -10px;
}

.cursor-coords {
  position: absolute;
  top: 12px;
  left: 8px;
  font-size: 0.625rem;
  color: var(--red-400);
  font-family: monospace;
}

.preview-info {
  display: flex;
  justify-content: space-between;
  padding-top: 8px;
  font-size: 0.6875rem;
  color: var(--text-color-secondary);
}

.detection-results {
  width: 300px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.results-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid var(--surface-border);
}

.results-header .title {
  font-weight: 600;
  font-size: 0.875rem;
}

.results-header .count {
  font-size: 0.6875rem;
  color: var(--text-color-secondary);
}

.results-list {
  flex: 1;
  overflow-y: auto;
}

.result-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  cursor: pointer;
  border-bottom: 1px solid var(--surface-border);
  transition: background 0.2s;
}

.result-item:hover {
  background: var(--surface-hover);
}

.result-item.selected {
  background: rgba(99, 102, 241, 0.1);
}

.result-icon {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  color: white;
  flex-shrink: 0;
}

.result-info {
  flex: 1;
  min-width: 0;
}

.result-name {
  font-size: 0.8125rem;
  font-weight: 500;
}

.result-meta {
  display: flex;
  gap: 12px;
  font-size: 0.625rem;
  color: var(--text-color-secondary);
  margin-top: 2px;
}

.result-confidence {
  text-align: right;
}

.confidence-value {
  font-size: 0.75rem;
  font-weight: 600;
}

.confidence-bar {
  width: 60px;
  height: 4px;
  background: var(--surface-border);
  border-radius: 2px;
  margin-top: 4px;
  overflow: hidden;
}

.confidence-fill {
  height: 100%;
  border-radius: 2px;
  transition: width 0.3s;
}

.no-results {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 32px;
  color: var(--text-color-secondary);
  gap: 8px;
}

.no-results i {
  font-size: 1.5rem;
  opacity: 0.5;
}

.selected-region-details {
  border-top: 1px solid var(--surface-border);
  padding: 12px 16px;
}

.details-header {
  font-size: 0.75rem;
  font-weight: 600;
  margin-bottom: 12px;
  color: var(--text-color-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.details-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
}

.detail-item {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.detail-item label {
  font-size: 0.625rem;
  color: var(--text-color-secondary);
  text-transform: uppercase;
}

.detail-item span {
  font-size: 0.75rem;
  font-family: monospace;
}

.details-actions {
  display: flex;
  gap: 8px;
  margin-top: 12px;
}

.details-actions :deep(.p-button) {
  flex: 1;
  font-size: 0.75rem;
}
</style>
