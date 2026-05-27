<template>
  <div class="node-config">
    <div class="config-section">
      <label class="config-label">节点名称</label>
      <InputText
        v-model="localConfig.label"
        @update:modelValue="updateConfig('label', $event)"
        placeholder="输入节点名称"
      />
    </div>

    <!-- Start Node Config -->
    <template v-if="node.type === 'start'">
      <div class="config-section">
        <label class="config-label">捕获区域</label>
        <Select
          v-model="localConfig.captureRegion"
          :options="captureRegionOptions"
          optionLabel="label"
          optionValue="value"
          placeholder="全屏"
          @update:modelValue="updateConfig('captureRegion', $event)"
        />
      </div>

      <div class="config-section">
        <label class="config-label">图像格式</label>
        <Select
          v-model="localConfig.imageFormat"
          :options="imageFormatOptions"
          optionLabel="label"
          optionValue="value"
          placeholder="PNG"
          @update:modelValue="updateConfig('imageFormat', $event)"
        />
      </div>
    </template>

    <!-- Find Image Config -->
    <template v-if="node.type === 'find-image'">
      <div class="config-section">
        <label class="config-label">目标图像</label>
        <Select
          v-model="localConfig.targetId"
          :options="imageTargetOptions"
          optionLabel="name"
          optionValue="id"
          placeholder="选择图像目标"
          @update:modelValue="updateConfig('targetId', $event)"
        />
      </div>

    </template>

    <!-- Click Config -->
    <template v-if="node.type === 'click' || node.type === 'double-click'">
      <div class="config-section">
        <label class="config-label">点击类型</label>
        <SelectButton
          v-model="localConfig.button"
          :options="buttonOptions"
          optionLabel="label"
          optionValue="value"
          @update:modelValue="updateConfig('button', $event)"
        />
      </div>

      <div class="config-section">
        <label class="config-label">点击位置偏移</label>
        <div class="offset-inputs">
          <div class="offset-field">
            <label>X</label>
            <InputNumber
              v-model="localConfig.offsetX"
              :step="1"
              @update:modelValue="updateConfig('offsetX', $event)"
            />
          </div>
          <div class="offset-field">
            <label>Y</label>
            <InputNumber
              v-model="localConfig.offsetY"
              :step="1"
              @update:modelValue="updateConfig('offsetY', $event)"
            />
          </div>
        </div>
      </div>

      <div class="config-section">
        <Checkbox
          v-model="localConfig.moveFirst"
          :binary="true"
          inputId="moveFirst"
          @update:modelValue="updateConfig('moveFirst', $event)"
        />
        <label for="moveFirst" class="checkbox-label">点击前移动鼠标</label>
      </div>
    </template>

    <!-- Type Text Config -->
    <template v-if="node.type === 'type-text'">
      <div class="config-section">
        <label class="config-label">输入文本</label>
        <Textarea
          v-model="localConfig.text"
          rows="3"
          placeholder="要输入的文本内容..."
          @update:modelValue="updateConfig('text', $event)"
        />
      </div>

      <div class="config-section">
        <label class="config-label">输入延迟 (ms)</label>
        <div class="slider-row">
          <Slider
            v-model="localConfig.delay"
            :min="0"
            :max="200"
            @update:modelValue="updateConfig('delay', $event)"
          />
          <span class="slider-value">{{ localConfig.delay || 0 }}ms</span>
        </div>
      </div>

      <div class="config-section">
        <Checkbox
          v-model="localConfig.clearFirst"
          :binary="true"
          inputId="clearFirst"
          @update:modelValue="updateConfig('clearFirst', $event)"
        />
        <label for="clearFirst" class="checkbox-label">输入前清空</label>
      </div>
    </template>

    <!-- Hotkey Config -->
    <template v-if="node.type === 'hotkey'">
      <div class="config-section">
        <label class="config-label">组合键</label>
        <div class="hotkey-builder">
          <div class="modifier-keys">
            <ToggleButton
              v-model="localConfig.ctrl"
              onLabel="Ctrl"
              offLabel="Ctrl"
              @update:modelValue="updateConfig('ctrl', $event)"
            />
            <ToggleButton
              v-model="localConfig.alt"
              onLabel="Alt"
              offLabel="Alt"
              @update:modelValue="updateConfig('alt', $event)"
            />
            <ToggleButton
              v-model="localConfig.shift"
              onLabel="Shift"
              offLabel="Shift"
              @update:modelValue="updateConfig('shift', $event)"
            />
            <ToggleButton
              v-model="localConfig.meta"
              onLabel="Meta"
              offLabel="Meta"
              @update:modelValue="updateConfig('meta', $event)"
            />
          </div>
          <InputText
            v-model="localConfig.key"
            placeholder="按键 (如: A, Enter, F1)"
            @update:modelValue="updateConfig('key', $event)"
          />
        </div>
        <div class="hotkey-preview">
          {{ hotkeyPreview }}
        </div>
      </div>
    </template>

    <!-- Wait Config -->
    <template v-if="node.type === 'wait'">
      <div class="config-section">
        <label class="config-label">等待时间 (ms)</label>
        <InputNumber
          v-model="localConfig.duration"
          :min="0"
          :max="300000"
          :step="100"
          @update:modelValue="updateConfig('duration', $event)"
        />
      </div>

      <div class="preset-buttons">
        <Button label="100ms" size="small" outlined @click="setDuration(100)" />
        <Button label="500ms" size="small" outlined @click="setDuration(500)" />
        <Button label="1s" size="small" outlined @click="setDuration(1000)" />
        <Button label="3s" size="small" outlined @click="setDuration(3000)" />
        <Button label="5s" size="small" outlined @click="setDuration(5000)" />
      </div>
    </template>

    <!-- Condition Config -->
    <template v-if="node.type === 'condition'">
      <div class="config-section">
        <label class="config-label">条件类型</label>
        <Select
          v-model="localConfig.conditionType"
          :options="conditionTypeOptions"
          optionLabel="label"
          optionValue="value"
          @update:modelValue="updateConfig('conditionType', $event)"
        />
      </div>

      <div class="config-section" v-if="localConfig.conditionType === 'image'">
        <label class="config-label">目标图像</label>
        <Select
          v-model="localConfig.targetId"
          :options="imageTargetOptions"
          optionLabel="name"
          optionValue="id"
          placeholder="选择图像目标"
          @update:modelValue="updateConfig('targetId', $event)"
        />
      </div>

      <div class="config-section">
        <label class="config-label">判断条件</label>
        <Select
          v-model="localConfig.operator"
          :options="operatorOptions"
          optionLabel="label"
          optionValue="value"
          @update:modelValue="updateConfig('operator', $event)"
        />
      </div>
    </template>

    <!-- Loop Config -->
    <template v-if="node.type === 'loop'">
      <div class="config-section">
        <label class="config-label">循环类型</label>
        <Select
          v-model="localConfig.loopType"
          :options="loopTypeOptions"
          optionLabel="label"
          optionValue="value"
          @update:modelValue="updateConfig('loopType', $event)"
        />
      </div>

      <div class="config-section" v-if="localConfig.loopType === 'count'">
        <label class="config-label">循环次数</label>
        <InputNumber
          v-model="localConfig.maxIterations"
          :min="1"
          :max="1000"
          @update:modelValue="updateConfig('maxIterations', $event)"
        />
      </div>

      <div class="config-section" v-if="localConfig.loopType === 'while'">
        <label class="config-label">循环条件</label>
        <Select
          v-model="localConfig.targetId"
          :options="imageTargetOptions"
          optionLabel="name"
          optionValue="id"
          placeholder="选择图像目标"
          @update:modelValue="updateConfig('targetId', $event)"
        />
      </div>
    </template>

    <!-- Move Mouse Config -->
    <template v-if="node.type === 'move-mouse'">
      <div class="config-section">
        <label class="config-label">移动方式</label>
        <SelectButton
          v-model="localConfig.moveType"
          :options="moveTypeOptions"
          optionLabel="label"
          optionValue="value"
          @update:modelValue="updateConfig('moveType', $event)"
        />
      </div>

      <div class="config-section" v-if="localConfig.moveType === 'absolute'">
        <label class="config-label">目标坐标</label>
        <div class="offset-inputs">
          <div class="offset-field">
            <label>X</label>
            <InputNumber
              v-model="localConfig.x"
              :min="0"
              @update:modelValue="updateConfig('x', $event)"
            />
          </div>
          <div class="offset-field">
            <label>Y</label>
            <InputNumber
              v-model="localConfig.y"
              :min="0"
              @update:modelValue="updateConfig('y', $event)"
            />
          </div>
        </div>
      </div>

      <div class="config-section">
        <label class="config-label">移动速度</label>
        <div class="slider-row">
          <Slider
            v-model="localConfig.speed"
            :min="0"
            :max="100"
            @update:modelValue="updateConfig('speed', $event)"
          />
          <span class="slider-value">{{ localConfig.speed || 50 }}%</span>
        </div>
      </div>
    </template>

    <!-- Drag Config -->
    <template v-if="node.type === 'drag'">
      <div class="config-section">
        <label class="config-label">起始位置</label>
        <div class="offset-inputs">
          <div class="offset-field">
            <label>X</label>
            <InputNumber
              v-model="localConfig.startX"
              :min="0"
              @update:modelValue="updateConfig('startX', $event)"
            />
          </div>
          <div class="offset-field">
            <label>Y</label>
            <InputNumber
              v-model="localConfig.startY"
              :min="0"
              @update:modelValue="updateConfig('startY', $event)"
            />
          </div>
        </div>
      </div>

      <div class="config-section">
        <label class="config-label">目标位置</label>
        <div class="offset-inputs">
          <div class="offset-field">
            <label>X</label>
            <InputNumber
              v-model="localConfig.endX"
              :min="0"
              @update:modelValue="updateConfig('endX', $event)"
            />
          </div>
          <div class="offset-field">
            <label>Y</label>
            <InputNumber
              v-model="localConfig.endY"
              :min="0"
              @update:modelValue="updateConfig('endY', $event)"
            />
          </div>
        </div>
      </div>
    </template>

    <!-- Advanced Options -->
    <div class="config-section advanced-toggle">
      <Button
        :label="showAdvanced ? '隐藏高级选项' : '显示高级选项'"
        icon="pi pi-sliders-h"
        text
        size="small"
        @click="showAdvanced = !showAdvanced"
      />
    </div>

    <template v-if="showAdvanced">
      <div class="config-section">
        <Checkbox
          v-model="localConfig.continueOnError"
          :binary="true"
          inputId="continueOnError"
          @update:modelValue="updateConfig('continueOnError', $event)"
        />
        <label for="continueOnError" class="checkbox-label">出错时继续</label>
      </div>

      <div class="config-section">
        <label class="config-label">重试次数</label>
        <InputNumber
          v-model="localConfig.retryCount"
          :min="0"
          :max="10"
          @update:modelValue="updateConfig('retryCount', $event)"
        />
      </div>

      <div class="config-section">
        <label class="config-label">备注</label>
        <Textarea
          v-model="localConfig.notes"
          rows="2"
          placeholder="添加备注..."
          @update:modelValue="updateConfig('notes', $event)"
        />
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, reactive } from "vue";
import InputText from "primevue/inputtext";
import InputNumber from "primevue/inputnumber";
import Textarea from "primevue/textarea";
import Select from "primevue/select";
import SelectButton from "primevue/selectbutton";
import Slider from "primevue/slider";
import Checkbox from "primevue/checkbox";
import ToggleButton from "primevue/togglebutton";
import Button from "primevue/button";
import type { WorkflowNode, ImageTarget } from "../types";

const props = defineProps<{
  node: WorkflowNode;
  imageTargets: ImageTarget[];
}>();

const emit = defineEmits<{
  update: [nodeId: string, config: Record<string, unknown>];
}>();

const showAdvanced = ref(false);

// Local config state
const localConfig = reactive<Record<string, unknown>>({
  label: props.node.label,
  ...props.node.config,
});

// Watch for node changes
watch(
  () => props.node,
  newNode => {
    Object.assign(localConfig, {
      label: newNode.label,
      ...newNode.config,
    });
  },
  { immediate: true },
);

// Options
const imageTargetOptions = computed(() => props.imageTargets);

const buttonOptions = [
  { label: "左键", value: "left" },
  { label: "中键", value: "middle" },
  { label: "右键", value: "right" },
];

const conditionTypeOptions = [
  { label: "图像存在", value: "image" },
  { label: "像素颜色", value: "pixel" },
  { label: "变量比较", value: "variable" },
];

const operatorOptions = [
  { label: "存在", value: "exists" },
  { label: "不存在", value: "not-exists" },
  { label: "等于", value: "equals" },
  { label: "不等于", value: "not-equals" },
];

const loopTypeOptions = [
  { label: "固定次数", value: "count" },
  { label: "条件循环", value: "while" },
  { label: "无限循环", value: "infinite" },
];

const moveTypeOptions = [
  { label: "绝对坐标", value: "absolute" },
  { label: "相对移动", value: "relative" },
  { label: "图像位置", value: "image" },
];

const captureRegionOptions = [
  { label: "全屏", value: "fullscreen" },
  { label: "主显示器", value: "primary" },
  { label: "活动窗口", value: "active-window" },
];

const imageFormatOptions = [
  { label: "PNG", value: "png" },
  { label: "JPEG", value: "jpeg" },
  { label: "BMP", value: "bmp" },
];

const hotkeyPreview = computed(() => {
  const parts: string[] = [];
  if (localConfig.ctrl) parts.push("Ctrl");
  if (localConfig.alt) parts.push("Alt");
  if (localConfig.shift) parts.push("Shift");
  if (localConfig.meta) parts.push("Meta");
  if (localConfig.key) parts.push(String(localConfig.key).toUpperCase());
  return parts.join(" + ") || "未设置";
});

const updateConfig = (key: string, value: unknown) => {
  localConfig[key] = value;
  emit("update", props.node.id, { [key]: value });
};

const setDuration = (ms: number) => {
  localConfig.duration = ms;
  emit("update", props.node.id, { duration: ms });
};
</script>

<style scoped>
.node-config {
  padding: 16px;
  overflow-y: auto;
  height: calc(100% - 65px);
}

.config-section {
  margin-bottom: 16px;
}

.config-label {
  display: block;
  font-size: 0.75rem;
  font-weight: 500;
  color: var(--text-color-secondary);
  margin-bottom: 6px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.config-section :deep(.p-inputtext),
.config-section :deep(.p-select),
.config-section :deep(.p-inputnumber),
.config-section :deep(.p-textarea) {
  width: 100%;
  font-size: 0.8125rem;
}

.slider-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.slider-row :deep(.p-slider) {
  flex: 1;
}

.slider-value {
  font-size: 0.75rem;
  font-family: monospace;
  color: var(--primary-color);
  min-width: 50px;
  text-align: right;
}

.offset-inputs {
  display: flex;
  gap: 12px;
}

.offset-field {
  flex: 1;
}

.offset-field label {
  display: block;
  font-size: 0.6875rem;
  color: var(--text-color-secondary);
  margin-bottom: 4px;
}

.checkbox-label {
  margin-left: 8px;
  font-size: 0.8125rem;
  color: var(--text-color);
  cursor: pointer;
}

.hotkey-builder {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.modifier-keys {
  display: flex;
  gap: 4px;
}

.modifier-keys :deep(.p-togglebutton) {
  flex: 1;
  padding: 6px;
  font-size: 0.6875rem;
}

.hotkey-preview {
  font-family: monospace;
  font-size: 0.875rem;
  padding: 8px 12px;
  background: var(--surface-hover);
  border-radius: 6px;
  text-align: center;
  color: var(--primary-color);
}

.preset-buttons {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: 8px;
}

.preset-buttons :deep(.p-button) {
  flex: 1;
  min-width: 60px;
}

.advanced-toggle {
  border-top: 1px solid var(--surface-border);
  padding-top: 16px;
  margin-top: 8px;
}

.advanced-toggle :deep(.p-button) {
  width: 100%;
  justify-content: center;
}
</style>
