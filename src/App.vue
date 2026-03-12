<script setup lang="ts">
import TopToolbar from "./components/TopToolbar.vue";
import UploadPanel from "./components/UploadPanel.vue";
import ResultPanel from "./components/ResultPanel.vue";
import LogPanel from "./components/LogPanel.vue";
</script>

<template>
  <div
    class="h-screen w-screen flex flex-col bg-slate-100 font-sans text-slate-800 overflow-hidden"
  >
    <!-- A. 顶部工具区 -->
    <TopToolbar />

    <!-- B + C 主体区域：左右分栏 -->
    <main class="flex-1 flex min-h-0 p-4 gap-4">
      <!-- B. 左侧上传区 -->
      <aside
        class="w-72 shrink-0 flex flex-col rounded-xl border border-slate-200 bg-white shadow-sm p-4"
      >
        <UploadPanel />
      </aside>

      <!-- C. 右侧结果展示区 -->
      <section
        class="flex-1 flex flex-col rounded-xl border border-slate-200 bg-white shadow-sm p-4 min-w-0"
      >
        <ResultPanel />
      </section>
    </main>

    <!-- D. 底部日志区 -->
    <footer
      class="h-40 shrink-0 mx-4 mb-4 rounded-xl border border-slate-200 bg-slate-800 shadow-sm p-4"
    >
      <LogPanel />
    </footer>
  </div>
</template>
