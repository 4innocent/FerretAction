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
  <section flex="~ col" h="full">
    <div flex="~" items="center" justify="between" m="b-3">
      <h2 text="sm slate-500" font="semibold" tracking="wide" uppercase>
        屏幕预览 & 识别结果
      </h2>
      <button
        rounded="lg"
        p="x-3 y-1"
        text="xs slate-700"
        font="medium"
        bg="slate-100 hover:slate-200"
        border="~ slate-300"
        transition="~"
        @click="captureScreen"
      >
        {{ screenImage ? "刷新屏幕" : "获取屏幕截图" }}
      </button>
    </div>

    <!-- 屏幕图展示区 -->
    <div
      flex="1 ~"
      items="center"
      justify="center"
      relative
      rounded="xl"
      overflow="hidden"
      bg="slate-50"
      border="~ slate-200"
      min-h="0"
    >
      <!-- 空状态 -->
      <div
        v-if="!screenImage"
        flex="~ col"
        items="center"
        gap="2"
        text="slate-400"
      >
        <div text="4xl">&#128421;</div>
        <p text="sm">请先获取当前屏幕内容</p>
      </div>

      <!-- 屏幕图 -->
      <template v-else>
        <img
          ref="imgRef"
          :src="screenImage"
          alt="当前屏幕"
          max-w="full"
          max-h="full"
          object="contain"
        />
        <!-- 识别框 -->
        <div
          v-if="
            boxStyle && (status === 'success' || status === 'low-confidence')
          "
          absolute
          pointer-events="none"
          border="2"
          rounded="sm"
          :class="
            status === 'success' ? 'border-emerald-500' : 'border-orange-400'
          "
          :style="boxStyle"
        >
          <span
            absolute
            top="-5"
            left="0"
            text="[10px] white"
            font="semibold"
            p="x-1"
            rounded="~"
            :class="status === 'success' ? 'bg-emerald-500' : 'bg-orange-400'"
          >
            识别结果
          </span>
        </div>
        <!-- 识别中遮罩 -->
        <div
          v-if="status === 'running'"
          absolute
          inset="0"
          bg="black/20"
          flex="~"
          items="center"
          justify="center"
          backdrop-blur="sm"
          rounded="2xl"
        >
          <span text="white sm" font="medium" animate="pulse"> 正在识别… </span>
        </div>
      </template>
    </div>

    <!-- 结果信息 -->
    <div m="t-3" text="xs" space="y-1.5">
      <!-- 成功 / 低可信度 -->
      <template
        v-if="result && (status === 'success' || status === 'low-confidence')"
      >
        <div
          grid="~ cols-2"
          gap="x-4 y-1"
          text="slate-600"
          bg="slate-50"
          rounded="lg"
          p="3"
          border="~ slate-200"
        >
          <span text="xs slate-400" font="medium" uppercase>区域坐标</span>
          <span>X {{ result.x }}，Y {{ result.y }}</span>
          <span text="xs slate-400" font="medium" uppercase>区域尺寸</span>
          <span>{{ result.width }} × {{ result.height }}</span>
          <span text="xs slate-400" font="medium" uppercase>匹配程度</span>
          <span>{{
            matchLevelLabel[result.matchLevel] ?? result.matchLevel
          }}</span>
          <span text="xs slate-400" font="medium" uppercase>可信度</span>
          <span>{{ (result.confidence * 100).toFixed(0) }}%</span>
          <span text="xs slate-400" font="medium" uppercase>耗时</span>
          <span>{{ result.duration }} ms</span>
        </div>
      </template>

      <!-- 失败提示 -->
      <div v-else-if="status === 'failed'" text="red-500">
        未在当前屏幕中找到有效匹配位置，请尝试刷新屏幕或更换截图。
      </div>
    </div>

    <input
      ref="screenFileInput"
      type="file"
      accept="image/*"
      hidden
      @change="handleScreenFile"
    />
  </section>
</template>
