<script setup lang="ts">
import { ref, watch, nextTick } from "vue";
import { useRecognition } from "../composables/useRecognition";

const { logs } = useRecognition();
const scrollEl = ref<HTMLDivElement | null>(null);

watch(
  () => logs.value.length,
  async () => {
    await nextTick();
    if (scrollEl.value) {
      scrollEl.value.scrollTop = scrollEl.value.scrollHeight;
    }
  },
);

const typeClass: Record<string, string> = {
  info: "text-slate-400",
  success: "text-emerald-400",
  error: "text-red-400",
  warn: "text-orange-400",
};
</script>

<template>
  <section class="flex flex-col h-full">
    <h2
      class="text-sm font-semibold tracking-wide text-slate-400 uppercase mb-2"
    >
      操作日志
    </h2>
    <div
      ref="scrollEl"
      class="flex-1 overflow-y-auto font-mono text-xs leading-relaxed space-y-0.5 min-h-0"
    >
      <p v-if="logs.length === 0" class="text-slate-500 italic">暂无记录</p>
      <p
        v-for="log in logs"
        :key="log.id"
        :class="typeClass[log.type] ?? 'text-ink-400'"
      >
        <span class="text-slate-500 mr-2 select-none">[{{ log.time }}]</span>
        {{ log.message }}
      </p>
    </div>
  </section>
</template>
