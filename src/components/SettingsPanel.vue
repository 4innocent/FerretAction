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
                :modelValue="theme"
                :options="themeOptions"
                optionLabel="label"
                optionValue="value"
                @update:modelValue="emit('update:theme', $event)"
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
              <InputNumber
                v-model="settings.autoSaveInterval"
                :min="10"
                :max="300"
                suffix=" 秒"
              />
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
              <InputNumber
                v-model="settings.maxLogs"
                :min="100"
                :max="10000"
                :step="100"
              />
            </div>
          </div>
        </TabPanel>

        <!-- Execution Settings -->
        <TabPanel value="execution">
          <div class="settings-section">
            <h3>执行设置</h3>
            <div class="setting-item">
              <div class="setting-info">
                <label>步骤延迟</label>
                <span>每个步骤之间的默认延迟（毫秒）</span>
              </div>
              <InputNumber
                :modelValue="stepDelay"
                :min="0"
                :max="5000"
                :step="50"
                suffix=" ms"
                @update:modelValue="emit('update:stepDelay', $event)"
              />
            </div>

            <div class="setting-item">
              <div class="setting-info">
                <label>停止策略</label>
                <span>执行过程中如何停止工作流</span>
              </div>
              <Select
                :modelValue="stopStrategy"
                :options="stopStrategyOptions"
                optionLabel="label"
                optionValue="value"
                @update:modelValue="emit('update:stopStrategy', $event)"
              />
            </div>

            <div class="setting-item">
              <div class="setting-info">
                <label>执行时最小化</label>
                <span>开始执行前自动最小化主窗口</span>
              </div>
              <ToggleSwitch
                :modelValue="minimizeOnExecute"
                @update:modelValue="emit('update:minimizeOnExecute', $event)"
              />
            </div>
          </div>

          <div class="settings-section">
            <h3>鼠标设置</h3>
            <div class="setting-item">
              <div class="setting-info">
                <label>鼠标移动时长</label>
                <span>拟人化轨迹最大时长，0 为瞬间直达</span>
              </div>
              <Select
                :modelValue="mouseDuration"
                :options="durationOptions"
                optionLabel="label"
                optionValue="value"
                @update:modelValue="emit('update:mouseDuration', $event)"
              />
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
                  <span
                    >任意时刻按下快捷键，捕获当前鼠标坐标并复制到粘贴板 (x,
                    y)</span
                  >
                </div>
                <div class="hotkey-input">
                  <InputText
                    :value="displayShortcut"
                    :placeholder="
                      recordingTarget === 'capture'
                        ? '按下快捷键...'
                        : '点击设置'
                    "
                    readonly
                    :class="{ recording: recordingTarget === 'capture' }"
                    @click="startRecording('capture')"
                    @keydown="onKeyDown"
                    @blur="stopRecording"
                  />
                </div>
                <div v-if="lastCapture" class="captured-result">
                  <span>最近捕获：{{ lastCapture }}</span>
                </div>
              </div>

              <div class="hotkey-item">
                <div class="hotkey-info">
                  <label>快捷执行</label>
                  <span>按下快捷键执行当前选中的工作流</span>
                </div>
                <div class="hotkey-input">
                  <InputText
                    :value="quickExecuteDisplay"
                    :placeholder="
                      recordingTarget === 'quickExecute'
                        ? '按下快捷键...'
                        : '点击设置'
                    "
                    readonly
                    :class="{ recording: recordingTarget === 'quickExecute' }"
                    @click="startRecording('quickExecute')"
                    @keydown="onKeyDown"
                    @blur="stopRecording"
                  />
                </div>
              </div>
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
import { ref, reactive } from "vue";
import Tabs from "primevue/tabs";
import TabList from "primevue/tablist";
import Tab from "primevue/tab";
import TabPanels from "primevue/tabpanels";
import TabPanel from "primevue/tabpanel";
import Select from "primevue/select";
import InputNumber from "primevue/inputnumber";
import InputText from "primevue/inputtext";
import ToggleSwitch from "primevue/toggleswitch";
import Button from "primevue/button";

const props = defineProps<{
  captureShortcut: string;
  quickExecuteShortcut: string;
  lastCapture: string;
  theme: string;
  mouseDuration: number;
  stepDelay: number;
  stopStrategy: string;
  minimizeOnExecute: boolean;
}>();

const emit = defineEmits<{
  "update:captureShortcut": [value: string];
  "update:quickExecuteShortcut": [value: string];
  "update:theme": [value: string];
  "update:mouseDuration": [value: number];
  "update:stepDelay": [value: number];
  "update:stopStrategy": [value: string];
  "update:minimizeOnExecute": [value: boolean];
}>();

const activeTab = ref("general");

const settings = reactive({
  language: "zh-CN",
  autoSave: true,
  autoSaveInterval: 60,
  logLevel: "info",
  maxLogs: 1000,
});

const recordingTarget = ref<string | null>(null);
const displayShortcut = ref(props.captureShortcut);
const quickExecuteDisplay = ref(props.quickExecuteShortcut);

function startRecording(target: string) {
  recordingTarget.value = target;
  if (target === "capture") displayShortcut.value = "";
  else quickExecuteDisplay.value = "";
  window.addEventListener("keydown", onKeyDown, true);
}

function stopRecording() {
  if (recordingTarget.value === "capture")
    displayShortcut.value = props.captureShortcut;
  else quickExecuteDisplay.value = props.quickExecuteShortcut;
  recordingTarget.value = null;
  window.removeEventListener("keydown", onKeyDown, true);
}

function onKeyDown(e: KeyboardEvent) {
  if (!recordingTarget.value) return;
  e.preventDefault();
  e.stopPropagation();

  const parts: string[] = [];
  if (e.ctrlKey || e.metaKey) parts.push("Ctrl");
  if (e.shiftKey) parts.push("Shift");
  if (e.altKey) parts.push("Alt");

  const key = e.key;
  if (["Control", "Shift", "Alt", "Meta"].includes(key)) return;

  const keyMap: Record<string, string> = {
    ArrowUp: "Up",
    ArrowDown: "Down",
    ArrowLeft: "Left",
    ArrowRight: "Right",
    Escape: "Esc",
    Insert: "Ins",
    Delete: "Del",
    PageUp: "PgUp",
    PageDown: "PgDn",
  };
  const displayKey =
    keyMap[key] || (key.length === 1 ? key.toUpperCase() : key);
  parts.push(displayKey);

  window.removeEventListener("keydown", onKeyDown, true);
  const newShortcut = parts.join("+");
  const target = recordingTarget.value;
  if (target === "capture") {
    displayShortcut.value = newShortcut;
    emit("update:captureShortcut", newShortcut);
  } else {
    quickExecuteDisplay.value = newShortcut;
    emit("update:quickExecuteShortcut", newShortcut);
  }
  recordingTarget.value = null;
}

const themeOptions = [
  { label: "深色模式", value: "dark" },
  { label: "浅色模式", value: "light" },
  { label: "跟随系统", value: "system" },
];

const languageOptions = [
  { label: "简体中文", value: "zh-CN" },
  // { label: 'English', value: 'en-US' }
];

const logLevelOptions = [
  { label: "调试", value: "debug" },
  { label: "信息", value: "info" },
  { label: "警告", value: "warning" },
  { label: "错误", value: "error" },
];

const durationOptions = Array.from({ length: 11 }, (_, i) => {
  const ms = i * 50;
  return { label: ms === 0 ? "瞬间 (0ms)" : `${ms}ms`, value: ms };
});

const stopStrategyOptions = [
  { label: "不停止", value: "none" },
  { label: "检测到鼠标移动时停止", value: "mouse" },
  { label: "按下 ESC 停止", value: "esc" },
];
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
  display: grid;
  grid-template-columns: 1fr 140px;
  align-items: center;
  gap: 12px;
  padding: 8px 0;
}

.setting-item:not(:last-child) {
  border-bottom: 1px solid var(--surface-border);
}

.setting-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  flex: 1;
  min-width: 0;
  padding-right: 16px;
}

.setting-info label {
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--text-color);
}

.setting-info span {
  font-size: 0.6875rem;
  color: var(--text-color-secondary);
  line-height: 1.35;
}

.setting-item :deep(.p-select) {
  width: 100%;
}

.setting-item :deep(.p-inputnumber) {
  display: flex;
  width: 100%;
}

.setting-item :deep(.p-inputnumber-suffix) {
  flex-shrink: 0;
}

.setting-item :deep(.p-inputnumber-input) {
  flex: 1;
  min-width: 0;
  padding: 6px 4px;
  font-size: 0.75rem;
}

.setting-item :deep(.p-toggleswitch) {
  justify-self: end;
}

.slider-setting {
  display: flex;
  align-items: center;
  gap: 6px;
}

.slider-setting :deep(.p-slider) {
  flex: 1;
}

.slider-setting .slider-value {
  font-size: 0.6875rem;
  font-family: monospace;
  color: var(--primary-color);
  width: 32px;
  text-align: right;
  flex-shrink: 0;
}

.hotkey-list {
  display: flex;
  flex-direction: column;
}

.hotkey-item {
  display: grid;
  grid-template-columns: 1fr 140px;
  align-items: center;
  gap: 12px;
  padding: 8px 0;
  border-bottom: 1px solid var(--surface-border);
}

.hotkey-info {
  display: flex;
  flex-direction: column;
  gap: 1px;
  flex: 1;
  min-width: 0;
  padding-right: 16px;
}

.hotkey-info label {
  font-size: 0.8125rem;
  font-weight: 500;
}

.hotkey-info span {
  font-size: 0.6875rem;
  color: var(--text-color-secondary);
  line-height: 1.35;
}

.hotkey-input {
  display: flex;
  align-items: center;
  gap: 4px;
}

.hotkey-input :deep(.p-inputtext) {
  width: 100%;
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
