<script setup lang="ts">
import { useRecognition } from "../composables/useRecognition";

const {
  status,
  statusLabel,
  canStartRecognition,
  startRecognition,
  clearAll,
  reRecognize,
  result,
} = useRecognition();
</script>

<template>
  <header
    flex="~"
    items="center"
    justify="between"
    p="x-6 y-3"
    bg="white"
    border="b slate-200"
    shadow="sm"
  >
    <!-- 左侧：标题 + 状态 -->
    <div flex="~" items="center" gap="4">
      <h1 text="base slate-800" font="bold" tracking="wide">
        局部截图位置识别 · 测试台
      </h1>
      <span
        inline-flex="~"
        items="center"
        gap="1.5"
        rounded="full"
        p="x-3 y-0.5"
        text="xs"
        font="semibold"
        border="~"
        :class="{
          'bg-slate-50 text-slate-400 border-slate-200': status === 'idle',
          'bg-blue-50 text-blue-600 border-blue-200': status === 'ready',
          'bg-amber-50 text-amber-600 border-amber-200': status === 'running',
          'bg-emerald-50 text-emerald-600 border-emerald-200':
            status === 'success',
          'bg-red-50 text-red-600 border-red-200': status === 'failed',
          'bg-orange-50 text-orange-600 border-orange-200':
            status === 'low-confidence',
        }"
      >
        <span
          h="1.5"
          w="1.5"
          rounded="full"
          :class="{
            'bg-slate-300': status === 'idle',
            'bg-blue-500': status === 'ready',
            'bg-amber-500 animate-pulse': status === 'running',
            'bg-emerald-500': status === 'success',
            'bg-red-500': status === 'failed',
            'bg-orange-500': status === 'low-confidence',
          }"
        />
        {{ statusLabel }}
      </span>
    </div>

    <!-- 右侧：操作按钮 -->
    <div flex="~" items="center" gap="2">
      <button
        rounded="lg"
        p="x-4 y-1.5"
        text="sm"
        font="medium"
        transition="~"
        :class="
          canStartRecognition
            ? 'bg-blue-600 text-white hover:bg-blue-700 active:bg-blue-800 shadow-sm'
            : 'bg-slate-100 text-slate-300 cursor-not-allowed'
        "
        :disabled="!canStartRecognition"
        @click="startRecognition"
      >
        开始识别
      </button>

      <button
        rounded="lg"
        p="x-4 y-1.5"
        text="sm"
        font="medium"
        transition="~"
        border="~"
        :class="
          result
            ? 'bg-white text-slate-700 border-slate-300 hover:bg-slate-50'
            : 'bg-slate-100 text-slate-300 border-slate-100 cursor-not-allowed'
        "
        :disabled="!result"
        @click="reRecognize"
      >
        重新识别
      </button>

      <button
        rounded="lg"
        p="x-4 y-1.5"
        text="sm slate-600"
        font="medium"
        bg="white hover:slate-50"
        border="~ slate-300"
        transition="~"
        :disabled="status === 'running'"
        @click="clearAll"
      >
        清空
      </button>
    </div>
  </header>
</template>
