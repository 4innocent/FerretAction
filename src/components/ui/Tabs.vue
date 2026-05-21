<script setup lang="ts">
import { provide, ref, readonly } from 'vue'

// ----------------------------------------------------------
// 标签页组件
// 用法:
//   <Tabs v-model="activeTab">
//     <TabsList>
//       <TabsTrigger value="tab1">Tab 1</TabsTrigger>
//     </TabsList>
//     <TabsContent value="tab1">Content</TabsContent>
//   </Tabs>
// ----------------------------------------------------------

const props = withDefaults(defineProps<{
  modelValue: string
  defaultValue?: string
}>(), {
  defaultValue: '',
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const activeTab = ref(props.modelValue || props.defaultValue)
provide('tabsActiveValue', readonly(activeTab))

function setActiveTab(value: string) {
  activeTab.value = value
  emit('update:modelValue', value)
}
provide('tabsSetValue', setActiveTab)
</script>

<template>
  <div class="flex flex-col h-full">
    <slot />
  </div>
</template>
