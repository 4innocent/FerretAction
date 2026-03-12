<script setup lang="ts">
import TopToolbar from "./components/TopToolbar.vue";
import UploadPanel from "./components/UploadPanel.vue";
import ResultPanel from "./components/ResultPanel.vue";
import LogPanel from "./components/LogPanel.vue";
</script>

<template>
  <div
    h="screen"
    w="screen"
    flex="~ col"
    bg="slate-100"
    font="sans"
    text="slate-800"
    overflow="hidden"
  >
    <!-- A. 顶部工具区 -->
    <TopToolbar />

    <!-- B + C 主体区域：左右分栏 -->
    <main flex="1 ~" min-h="0" p="4" gap="4">
      <!-- B. 左侧上传区 -->
      <aside
        w="72"
        shrink="0"
        flex="~ col"
        rounded="xl"
        border="~ slate-200"
        bg="white"
        shadow="sm"
        p="4"
      >
        <UploadPanel />
      </aside>

      <!-- C. 右侧结果展示区 -->
      <section
        flex="1 ~ col"
        rounded="xl"
        border="~ slate-200"
        bg="white"
        shadow="sm"
        p="4"
        min-w="0"
      >
        <ResultPanel />
      </section>
    </main>

    <!-- D. 底部日志区 -->
    <footer
      h="40"
      shrink="0"
      m="x-4 b-4"
      rounded="xl"
      border="~ slate-200"
      bg="slate-800"
      shadow="sm"
      p="4"
    >
      <LogPanel />
    </footer>
  </div>
</template>
