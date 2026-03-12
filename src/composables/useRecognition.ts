import { ref, computed } from "vue";
import type {
  RecognitionStatus,
  RecognitionResult,
  UploadedImage,
  LogEntry,
} from "../types";

// ── 单例状态（组件间共享） ──
const uploadedImage = ref<UploadedImage | null>(null);
const screenImage = ref<string | null>(null); // dataUrl
const screenImageSize = ref<{ width: number; height: number } | null>(null);
const status = ref<RecognitionStatus>("idle");
const result = ref<RecognitionResult | null>(null);
const logs = ref<LogEntry[]>([]);

let logIdSeq = 0;

function addLog(message: string, type: LogEntry["type"] = "info") {
  logs.value.push({
    id: ++logIdSeq,
    time: new Date().toLocaleTimeString(),
    message,
    type,
  });
}

/** 状态文案映射 */
const statusLabel = computed(() => {
  const map: Record<RecognitionStatus, string> = {
    idle: "未开始",
    ready: "待识别",
    running: "识别中…",
    success: "识别成功",
    failed: "识别失败",
    "low-confidence": "结果可信度较低",
  };
  return map[status.value];
});

const canStartRecognition = computed(
  () =>
    uploadedImage.value !== null &&
    screenImage.value !== null &&
    status.value !== "running",
);

// ── 操作方法 ──

/** 设置上传图片 */
function setUploadedImage(img: UploadedImage) {
  uploadedImage.value = img;
  updateReadyStatus();
  addLog(`图片上传成功：${img.name}（${img.width}×${img.height}）`, "success");
}

/** 移除上传图片 */
function removeUploadedImage() {
  uploadedImage.value = null;
  status.value = "idle";
  result.value = null;
  addLog("已删除上传图片", "info");
}

/** 设置当前屏幕图 */
function setScreenImage(dataUrl: string, w: number, h: number) {
  screenImage.value = dataUrl;
  screenImageSize.value = { width: w, height: h };
  updateReadyStatus();
  addLog(`屏幕截图已更新（${w}×${h}）`, "success");
}

function updateReadyStatus() {
  if (
    uploadedImage.value &&
    screenImage.value &&
    status.value !== "running" &&
    status.value !== "success" &&
    status.value !== "low-confidence"
  ) {
    status.value = "ready";
  }
}

/** 模拟识别（后续接入真实后端） */
async function startRecognition() {
  if (!canStartRecognition.value) return;

  status.value = "running";
  result.value = null;
  addLog("开始识别…", "info");

  const start = performance.now();

  // 模拟耗时
  await new Promise(r => setTimeout(r, 800 + Math.random() * 1200));

  const duration = Math.round(performance.now() - start);

  // 模拟结果（70% 成功、15% 低可信度、15% 失败）
  const rand = Math.random();
  if (rand < 0.7) {
    const sw = screenImageSize.value?.width ?? 1920;
    const sh = screenImageSize.value?.height ?? 1080;
    const uw = uploadedImage.value!.width;
    const uh = uploadedImage.value!.height;
    const x = Math.round(Math.random() * (sw - uw));
    const y = Math.round(Math.random() * (sh - uh));
    result.value = {
      x,
      y,
      width: uw,
      height: uh,
      confidence: +(0.75 + Math.random() * 0.25).toFixed(2),
      matchLevel: "high",
      duration,
    };
    status.value = "success";
    addLog(
      `识别成功 — 位置 (${x}, ${y})，可信度 ${result.value.confidence}，耗时 ${duration}ms`,
      "success",
    );
  } else if (rand < 0.85) {
    const sw = screenImageSize.value?.width ?? 1920;
    const sh = screenImageSize.value?.height ?? 1080;
    const x = Math.round(Math.random() * sw * 0.6);
    const y = Math.round(Math.random() * sh * 0.6);
    result.value = {
      x,
      y,
      width: uploadedImage.value!.width,
      height: uploadedImage.value!.height,
      confidence: +(0.35 + Math.random() * 0.3).toFixed(2),
      matchLevel: "low",
      duration,
    };
    status.value = "low-confidence";
    addLog(
      `识别完成但可信度较低 — 可信度 ${result.value.confidence}，耗时 ${duration}ms`,
      "warn",
    );
  } else {
    status.value = "failed";
    addLog(
      `识别失败 — 未在当前屏幕中找到有效匹配，耗时 ${duration}ms`,
      "error",
    );
  }
}

/** 清空全部 */
function clearAll() {
  uploadedImage.value = null;
  screenImage.value = null;
  screenImageSize.value = null;
  status.value = "idle";
  result.value = null;
  addLog("已清空所有结果", "info");
}

/** 重新识别（保留素材） */
function reRecognize() {
  result.value = null;
  status.value = "ready";
  startRecognition();
}

export function useRecognition() {
  return {
    // state
    uploadedImage,
    screenImage,
    screenImageSize,
    status,
    result,
    logs,
    // computed
    statusLabel,
    canStartRecognition,
    // actions
    setUploadedImage,
    removeUploadedImage,
    setScreenImage,
    startRecognition,
    clearAll,
    reRecognize,
    addLog,
  };
}
