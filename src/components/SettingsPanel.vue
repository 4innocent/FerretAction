<template>
  <div class="settings-panel">
    <Tabs v-model:value="activeTab">
      <TabList>
        <Tab value="general">常规</Tab>
        <Tab value="execution">执行</Tab>
        <Tab value="hotkeys">快捷键</Tab>
        <Tab value="about">关于</Tab>
      </TabList>
      <TabPanels>
        <!-- General Settings -->
        <TabPanel value="general">
          <div class="settings-section">
            <h3>界面设置</h3>
            <div class="setting-item">
              <div class="setting-info">
                <label>主题</label>
                <span>选择应用界面主题</span>
              </div>
              <Select 
                v-model="settings.theme"
                :options="themeOptions"
                optionLabel="label"
                optionValue="value"
              />
            </div>
            
            <div class="setting-item">
              <div class="setting-info">
                <label>语言</label>
                <span>界面显示语言</span>
              </div>
              <Select 
                v-model="settings.language"
                :options="languageOptions"
                optionLabel="label"
                optionValue="value"
              />
            </div>
            
            <div class="setting-item">
              <div class="setting-info">
                <label>自动保存</label>
                <span>自动保存工作流更改</span>
              </div>
              <ToggleSwitch v-model="settings.autoSave" />
            </div>
            
            <div class="setting-item">
              <div class="setting-info">
                <label>自动保存间隔</label>
                <span>自动保存的时间间隔（秒）</span>
              </div>
              <InputNumber v-model="settings.autoSaveInterval" :min="10" :max="300" suffix=" 秒" />
            </div>
          </div>

          <div class="settings-section">
            <h3>日志设置</h3>
            <div class="setting-item">
              <div class="setting-info">
                <label>日志级别</label>
                <span>显示的最低日志级别</span>
              </div>
              <Select 
                v-model="settings.logLevel"
                :options="logLevelOptions"
                optionLabel="label"
                optionValue="value"
              />
            </div>
            
            <div class="setting-item">
              <div class="setting-info">
                <label>保留日志数量</label>
                <span>最多保留的日志条目数</span>
              </div>
              <InputNumber v-model="settings.maxLogs" :min="100" :max="10000" :step="100" />
            </div>
          </div>
        </TabPanel>

        <!-- Execution Settings -->
        <TabPanel value="execution">
          <div class="settings-section">
            <h3>执行设置</h3>
            <div class="setting-item">
              <div class="setting-info">
                <label>执行速度</label>
                <span>工作流执行的整体速度</span>
              </div>
              <div class="slider-setting">
                <Slider v-model="settings.executionSpeed" :min="0.25" :max="2" :step="0.25" />
                <span class="slider-value">{{ settings.executionSpeed }}x</span>
              </div>
            </div>
            
            <div class="setting-item">
              <div class="setting-info">
                <label>步骤延迟</label>
                <span>每个步骤之间的默认延迟（毫秒）</span>
              </div>
              <InputNumber v-model="settings.stepDelay" :min="0" :max="5000" :step="50" suffix=" ms" />
            </div>
            
            <div class="setting-item">
              <div class="setting-info">
                <label>错误重试次数</label>
                <span>发生错误时的默认重试次数</span>
              </div>
              <InputNumber v-model="settings.retryCount" :min="0" :max="10" />
            </div>
            
            <div class="setting-item">
              <div class="setting-info">
                <label>失败时停止</label>
                <span>遇到错误时停止执行</span>
              </div>
              <ToggleSwitch v-model="settings.stopOnError" />
            </div>
          </div>

          <div class="settings-section">
            <h3>鼠标设置</h3>
            <div class="setting-item">
              <div class="setting-info">
                <label>鼠标移动速度</label>
                <span>鼠标移动的默认速度</span>
              </div>
              <div class="slider-setting">
                <Slider v-model="settings.mouseSpeed" :min="0" :max="100" />
                <span class="slider-value">{{ settings.mouseSpeed }}%</span>
              </div>
            </div>
            
            <div class="setting-item">
              <div class="setting-info">
                <label>点击延迟</label>
                <span>点击后的默认等待时间（毫秒）</span>
              </div>
              <InputNumber v-model="settings.clickDelay" :min="0" :max="1000" :step="10" suffix=" ms" />
            </div>
          </div>
        </TabPanel>

        <!-- Hotkeys Settings -->
        <TabPanel value="hotkeys">
          <div class="settings-section">
            <h3>全局快捷键</h3>
            <div class="hotkey-list">
              <div class="hotkey-item">
                <div class="hotkey-info">
                  <label>捕获鼠标坐标</label>
                  <span>任意时刻按下快捷键，捕获当前鼠标坐标并复制到粘贴板 (x, y)</span>
                </div>
                <div class="hotkey-input">
                  <InputText
                    :value="displayShortcut"
                    :placeholder="recording ? '按下快捷键...' : '点击设置'"
                    readonly
                    :class="{ recording }"
                    @click="startRecording"
                    @keydown="onKeyDown"
                    @blur="stopRecording"
                  />
                </div>
              </div>
            </div>
            <div v-if="lastCapture" class="captured-result">
              <span>最近捕获：{{ lastCapture }}</span>
            </div>
          </div>
        </TabPanel>

        <!-- About -->
        <TabPanel value="about">
          <div class="about-content">
            <div class="about-logo">
              <i class="pi pi-bolt"></i>
              <span class="app-name">AutoVision</span>
            </div>
            <div class="about-version">版本 1.0.0</div>
            <div class="about-description">
              AutoVision 是一款强大的桌面自动化工具，通过图像识别和工作流编排，
              帮助您自动化重复性任务，提高工作效率。
            </div>
            <div class="about-links">
              <Button label="查看文档" icon="pi pi-book" outlined />
              <Button label="GitHub" icon="pi pi-github" outlined />
              <Button label="反馈问题" icon="pi pi-comment" outlined />
            </div>
            <div class="about-credits">
              <span>© 2024 AutoVision. 保留所有权利。</span>
            </div>
          </div>
        </TabPanel>
      </TabPanels>
    </Tabs>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue'
import Tabs from 'primevue/tabs'
import TabList from 'primevue/tablist'
import Tab from 'primevue/tab'
import TabPanels from 'primevue/tabpanels'
import TabPanel from 'primevue/tabpanel'
import Select from 'primevue/select'
import Slider from 'primevue/slider'
import InputNumber from 'primevue/inputnumber'
import InputText from 'primevue/inputtext'
import ToggleSwitch from 'primevue/toggleswitch'
import Button from 'primevue/button'

const props = defineProps<{
  shortcut: string;
  lastCapture: string;
}>();

const emit = defineEmits<{
  "update:shortcut": [value: string];
}>();

const activeTab = ref('general')

const settings = reactive({
  theme: 'dark',
  language: 'zh-CN',
  autoSave: true,
  autoSaveInterval: 60,
  logLevel: 'info',
  maxLogs: 1000,
  executionSpeed: 1,
  stepDelay: 100,
  retryCount: 3,
  stopOnError: true,
  mouseSpeed: 50,
  clickDelay: 50
})

const recording = ref(false);
const displayShortcut = ref(props.shortcut);

function startRecording() {
  recording.value = true;
  displayShortcut.value = "";
  window.addEventListener("keydown", onKeyDown, true);
}

function stopRecording() {
  recording.value = false;
  displayShortcut.value = props.shortcut;
  window.removeEventListener("keydown", onKeyDown, true);
}

function onKeyDown(e: KeyboardEvent) {
  if (!recording.value) return;
  e.preventDefault();
  e.stopPropagation();

  const parts: string[] = [];
  if (e.ctrlKey || e.metaKey) parts.push("Ctrl");
  if (e.shiftKey) parts.push("Shift");
  if (e.altKey) parts.push("Alt");

  const key = e.key;
  if (["Control", "Shift", "Alt", "Meta"].includes(key)) return;

  const keyMap: Record<string, string> = {
    ArrowUp: "Up", ArrowDown: "Down", ArrowLeft: "Left", ArrowRight: "Right",
    Escape: "Esc", Insert: "Ins", Delete: "Del", PageUp: "PgUp", PageDown: "PgDn",
  };
  const displayKey = keyMap[key] || (key.length === 1 ? key.toUpperCase() : key);
  parts.push(displayKey);

  window.removeEventListener("keydown", onKeyDown, true);
  const newShortcut = parts.join("+");
  displayShortcut.value = newShortcut;
  recording.value = false;
  emit("update:shortcut", newShortcut);
}

const themeOptions = [
  { label: '深色模式', value: 'dark' },
  { label: '浅色模式', value: 'light' },
  { label: '跟随系统', value: 'system' }
]

const languageOptions = [
  { label: '简体中文', value: 'zh-CN' },
  { label: 'English', value: 'en-US' }
]

const logLevelOptions = [
  { label: '调试', value: 'debug' },
  { label: '信息', value: 'info' },
  { label: '警告', value: 'warning' },
  { label: '错误', value: 'error' }
]
</script>

<style scoped>
.settings-panel {
  height: 400px;
  overflow: hidden;
}

.settings-panel :deep(.p-tabs) {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.settings-panel :deep(.p-tablist) {
  padding: 0 8px;
  background: transparent;
}

.settings-panel :deep(.p-tab) {
  padding: 10px 16px;
  font-size: 0.875rem;
}

.settings-panel :deep(.p-tabpanels) {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
}

.settings-panel :deep(.p-tabpanel) {
  padding: 0;
}

.settings-section {
  padding: 12px 16px;
  border-bottom: 1px solid var(--surface-border);
}

.settings-section h3 {
  font-size: 0.8125rem;
  font-weight: 600;
  color: var(--text-color);
  margin-bottom: 12px;
}

.setting-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 0;
}

.setting-item:not(:last-child) {
  border-bottom: 1px solid var(--surface-border);
}

.setting-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.setting-info label {
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--text-color);
}

.setting-info span {
  font-size: 0.6875rem;
  color: var(--text-color-secondary);
}

.setting-item :deep(.p-select) {
  width: 100px;
  flex-shrink: 0;
}

.setting-item :deep(.p-inputnumber) {
  width: 75px;
  flex-shrink: 0;
}

.setting-item :deep(.p-inputnumber-input) {
  padding: 6px 8px;
  font-size: 0.8125rem;
}

.slider-setting {
  display: flex;
  align-items: center;
  gap: 6px;
  width: 120px;
  flex-shrink: 0;
}

.slider-setting :deep(.p-slider) {
  flex: 1;
}

.slider-setting .slider-value {
  font-size: 0.6875rem;
  font-family: monospace;
  color: var(--primary-color);
  min-width: 32px;
  text-align: right;
}

.hotkey-list {
  display: flex;
  flex-direction: column;
}

.hotkey-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 0;
  border-bottom: 1px solid var(--surface-border);
}

.hotkey-info {
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.hotkey-info label {
  font-size: 0.8125rem;
  font-weight: 500;
}

.hotkey-info span {
  font-size: 0.6875rem;
  color: var(--text-color-secondary);
}

.hotkey-input {
  display: flex;
  align-items: center;
  gap: 4px;
}

.hotkey-input :deep(.p-inputtext) {
  width: 100px;
  text-align: center;
  font-family: monospace;
  font-size: 0.75rem;
  cursor: pointer;
}

.hotkey-input :deep(.p-inputtext.recording) {
  border-color: var(--primary-color);
  box-shadow: 0 0 0 1px var(--primary-color);
}

.captured-result {
  margin-top: 8px;
  font-size: 0.75rem;
  color: var(--text-color-secondary);
}

.about-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 24px 16px;
  text-align: center;
}

.about-logo {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 8px;
}

.about-logo i {
  font-size: 2rem;
  color: var(--primary-color);
}

.app-name {
  font-size: 1.5rem;
  font-weight: 700;
  background: linear-gradient(135deg, var(--primary-color), var(--cyan-400));
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.about-version {
  font-size: 0.8125rem;
  color: var(--text-color-secondary);
  margin-bottom: 12px;
}

.about-description {
  max-width: 320px;
  font-size: 0.8125rem;
  line-height: 1.5;
  color: var(--text-color-secondary);
  margin-bottom: 16px;
}

.about-links {
  display: flex;
  gap: 8px;
  margin-bottom: 20px;
}

.about-credits {
  font-size: 0.75rem;
  color: var(--text-color-secondary);
}
</style>
