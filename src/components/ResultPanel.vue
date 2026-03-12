<script setup lang="ts">
import { ref, computed } from "vue";
import { useRecognition } from "../composables/useRecognition";

const { screenImage, screenImageSize, status, result, setScreenImage, addLog } =
  useRecognition();

const imgRef = ref<HTMLImageElement | null>(null);

/** 模拟获取屏幕截图：让用户选一张图作为"屏幕图" */
const screenFileInput = ref<HTMLInputElement | null>(null);

function captureScreen() {
  screenFileInput.value?.click();
}

function handleScreenFile(e: Event) {
  const input = e.target as HTMLInputElement;
  const file = input.files?.[0];
  if (!file) return;
  input.value = "";
  const reader = new FileReader();
  reader.onload = () => {
    const dataUrl = reader.result as string;
    const img = new Image();
    img.onload = () => setScreenImage(dataUrl, img.width, img.height);
    img.onerror = () => addLog("屏幕截图读取失败", "error");
    img.src = dataUrl;
  };
  reader.readAsDataURL(file);
}

/** 计算框选在渲染图上的位置（按图片实际显示缩放比） */
const boxStyle = computed(() => {
  if (!result.value || !screenImageSize.value || !imgRef.value) return null;
  const img = imgRef.value;
  const natW = screenImageSize.value.width;
  const natH = screenImageSize.value.height;
  const dispW = img.clientWidth;
  const dispH = img.clientHeight;
  const scaleX = dispW / natW;
  const scaleY = dispH / natH;
  return {
    left: `${result.value.x * scaleX}px`,
    top: `${result.value.y * scaleY}px`,
    width: `${result.value.width * scaleX}px`,
    height: `${result.value.height * scaleY}px`,
  };
});

const matchLevelLabel: Record<string, string> = {
  high: "高",
  medium: "中",
  low: "低",
};
</script>

<template>
  <section class="flex flex-col h-full">
    <div class="flex items-center justify-between mb-3">
      <h2 class="text-sm font-semibold tracking-wide text-slate-500 uppercase">
        屏幕预览 & 识别结果
      </h2>
      <button
        class="rounded-lg px-3 py-1 text-xs font-medium bg-slate-100 text-slate-700 border border-slate-300 hover:bg-slate-200 transition"
        @click="captureScreen"
      >
        {{ screenImage ? "刷新屏幕" : "获取屏幕截图" }}
      </button>
    </div>

    <!-- 屏幕图展示区 -->
    <div
      class="flex-1 relative rounded-xl overflow-hidden bg-slate-50 border border-slate-200 min-h-0 flex items-center justify-center"
    >
      <!-- 空状态 -->
      <div
        v-if="!screenImage"
        class="flex flex-col items-center gap-2 text-slate-400"
      >
        <div class="text-4xl">&#128421;</div>
        <p class="text-sm">请先获取当前屏幕内容</p>
      </div>

      <!-- 屏幕图 -->
      <template v-else>
        <img
          ref="imgRef"
          :src="screenImage"
          alt="当前屏幕"
          class="max-w-full max-h-full object-contain"
        />
        <!-- 识别框 -->
        <div
          v-if="
            boxStyle && (status === 'success' || status === 'low-confidence')
          "
          class="absolute pointer-events-none border-2 rounded-sm"
          :class="
            status === 'success' ? 'border-emerald-500' : 'border-orange-400'
          "
          :style="boxStyle"
        >
          <span
            class="absolute -top-5 left-0 text-[10px] font-semibold px-1 rounded"
            :class="
              status === 'success'
                ? 'bg-emerald-500 text-white'
                : 'bg-orange-400 text-white'
            "
          >
            识别结果
          </span>
        </div>
        <!-- 识别中遮罩 -->
        <div
          v-if="status === 'running'"
          class="absolute inset-0 bg-black/20 flex items-center justify-center backdrop-blur-sm rounded-2xl"
        >
          <span class="text-white text-sm font-medium animate-pulse"
            >正在识别…</span
          >
        </div>
      </template>
    </div>

    <!-- 结果信息 -->
    <div class="mt-3 text-xs space-y-1.5">
      <!-- 成功 / 低可信度 -->
      <template
        v-if="result && (status === 'success' || status === 'low-confidence')"
      >
        <div
          class="grid grid-cols-2 gap-x-4 gap-y-1 text-slate-600 bg-slate-50 rounded-lg p-3 border border-slate-200"
        >
          <span class="text-xs font-medium text-slate-400 uppercase"
            >区域坐标</span
          >
          <span>X {{ result.x }}，Y {{ result.y }}</span>
          <span class="text-xs font-medium text-slate-400 uppercase"
            >区域尺寸</span
          >
          <span>{{ result.width }} × {{ result.height }}</span>
          <span class="text-xs font-medium text-slate-400 uppercase"
            >匹配程度</span
          >
          <span>{{
            matchLevelLabel[result.matchLevel] ?? result.matchLevel
          }}</span>
          <span class="text-xs font-medium text-slate-400 uppercase"
            >可信度</span
          >
          <span>{{ (result.confidence * 100).toFixed(0) }}%</span>
          <span class="text-xs font-medium text-slate-400 uppercase">耗时</span>
          <span>{{ result.duration }} ms</span>
        </div>
      </template>

      <!-- 失败提示 -->
      <div v-else-if="status === 'failed'" class="text-red-500">
        未在当前屏幕中找到有效匹配位置，请尝试刷新屏幕或更换截图。
      </div>
    </div>

    <input
      ref="screenFileInput"
      type="file"
      accept="image/*"
      class="hidden"
      @change="handleScreenFile"
    />
  </section>
</template>
