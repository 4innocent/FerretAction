<script setup lang="ts">
import { ref } from "vue";
import { useRecognition } from "../composables/useRecognition";
import type { UploadedImage } from "../types";

const { uploadedImage, setUploadedImage, removeUploadedImage, addLog } =
  useRecognition();

const fileInput = ref<HTMLInputElement | null>(null);
const isDragOver = ref(false);

const MAX_SIZE = 20 * 1024 * 1024; // 20 MB
const ALLOWED_TYPES = ["image/png", "image/jpeg", "image/webp", "image/bmp"];

function openFilePicker() {
  fileInput.value?.click();
}

function handleFileChange(e: Event) {
  const input = e.target as HTMLInputElement;
  if (input.files?.[0]) {
    processFile(input.files[0]);
    input.value = "";
  }
}

function handleDrop(e: DragEvent) {
  isDragOver.value = false;
  const file = e.dataTransfer?.files?.[0];
  if (file) processFile(file);
}

function processFile(file: File) {
  if (!ALLOWED_TYPES.includes(file.type)) {
    addLog(`不支持的文件格式：${file.type || "未知"}`, "error");
    return;
  }
  if (file.size > MAX_SIZE) {
    addLog(
      `图片过大（${(file.size / 1024 / 1024).toFixed(1)} MB），请上传 20 MB 以内的图片`,
      "error",
    );
    return;
  }

  const reader = new FileReader();
  reader.onload = () => {
    const dataUrl = reader.result as string;
    const img = new Image();
    img.onload = () => {
      const info: UploadedImage = {
        name: file.name,
        size: file.size,
        width: img.width,
        height: img.height,
        dataUrl,
      };
      setUploadedImage(info);
    };
    img.onerror = () => addLog("图片读取失败，文件可能已损坏", "error");
    img.src = dataUrl;
  };
  reader.onerror = () => addLog("文件读取失败", "error");
  reader.readAsDataURL(file);
}

function formatSize(bytes: number) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}
</script>

<template>
  <section class="flex flex-col h-full">
    <h2
      class="text-sm font-semibold tracking-wide text-slate-500 uppercase mb-3"
    >
      局部截图
    </h2>

    <!-- 上传区域 -->
    <div
      v-if="!uploadedImage"
      class="flex-1 flex flex-col items-center justify-center gap-3 rounded-xl border-2 border-dashed transition cursor-pointer select-none"
      :class="
        isDragOver
          ? 'border-blue-400 bg-blue-50'
          : 'border-slate-300 hover:border-blue-300 bg-slate-50'
      "
      @click="openFilePicker"
      @dragover.prevent="isDragOver = true"
      @dragleave="isDragOver = false"
      @drop.prevent="handleDrop"
    >
      <div class="text-3xl text-slate-300">&#128444;</div>
      <p class="text-sm text-slate-500">点击或拖拽上传局部截图</p>
      <p class="text-xs text-slate-400">支持 PNG / JPG / WebP / BMP</p>
    </div>

    <!-- 预览区域 -->
    <div v-else class="flex-1 flex flex-col gap-3 min-h-0">
      <div
        class="flex-1 relative rounded-xl overflow-hidden bg-slate-50 border border-slate-200 min-h-0 flex items-center justify-center"
      >
        <img
          :src="uploadedImage.dataUrl"
          :alt="uploadedImage.name"
          class="max-w-full max-h-full object-contain"
        />
      </div>

      <!-- 图片信息 -->
      <div class="text-xs text-slate-500 space-y-0.5">
        <p class="truncate" :title="uploadedImage.name">
          {{ uploadedImage.name }}
        </p>
        <p>
          {{ uploadedImage.width }} × {{ uploadedImage.height }} ·
          {{ formatSize(uploadedImage.size) }}
        </p>
      </div>

      <!-- 操作按钮 -->
      <div class="flex gap-2">
        <button
          class="flex-1 rounded-lg py-1.5 text-xs font-medium bg-slate-100 text-slate-700 border border-slate-200 hover:bg-slate-200 transition"
          @click="openFilePicker"
        >
          替换图片
        </button>
        <button
          class="flex-1 rounded-lg py-1.5 text-xs font-medium bg-red-50 text-red-500 hover:bg-red-100 transition"
          @click="removeUploadedImage"
        >
          删除
        </button>
      </div>
    </div>

    <input
      ref="fileInput"
      type="file"
      accept="image/png,image/jpeg,image/webp,image/bmp"
      class="hidden"
      @change="handleFileChange"
    />
  </section>
</template>
