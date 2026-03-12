/** 识别状态枚举 */
export type RecognitionStatus =
  | "idle" // 未开始
  | "ready" // 待识别（素材已就绪）
  | "running" // 识别中
  | "success" // 识别成功
  | "failed" // 识别失败
  | "low-confidence"; // 结果可信度较低

/** 匹配程度 */
export type MatchLevel = "high" | "medium" | "low";

/** 识别结果 */
export interface RecognitionResult {
  x: number;
  y: number;
  width: number;
  height: number;
  confidence: number;
  matchLevel: MatchLevel;
  duration: number; // 耗时 ms
}

/** 上传的图片信息 */
export interface UploadedImage {
  name: string;
  size: number; // bytes
  width: number;
  height: number;
  dataUrl: string;
}

/** 日志条目 */
export interface LogEntry {
  id: number;
  time: string;
  message: string;
  type: "info" | "success" | "error" | "warn";
}
