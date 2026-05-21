<script setup lang="ts">
import { ref } from 'vue'
import type { WorkflowNodeData } from '../../types'
import { NODE_TYPE_CONFIG } from '../../types'
import Button from '../ui/Button.vue'
import Input from '../ui/Input.vue'
import Textarea from '../ui/Textarea.vue'
import Switch from '../ui/Switch.vue'
import Slider from '../ui/Slider.vue'
import Separator from '../ui/Separator.vue'
import Select from '../ui/Select.vue'
import { Collapsible, CollapsibleTrigger, CollapsibleContent } from '../ui'
import DynamicIcon from '../DynamicIcon.vue'

// ============================================================
// 节点配置面板（右侧）
// 根据选中节点的类型展示对应的配置表单
// 每种节点类型有各自的配置区块，通过可折叠面板组织
// ============================================================

defineProps<{
  node: WorkflowNodeData | null
}>()

const emit = defineEmits<{
  update: [updates: Partial<WorkflowNodeData>]
  delete: []
  duplicate: []
}>()

// ---- 各类型节点的临时配置值 ----
const findMatchPrecision = ref([85])
const findSearchArea = ref('full')
const findGrayscale = ref(false)
const findTimeout = ref('10')
const findFailAction = ref('continue')

const mouseMode = ref('absolute')
const mouseX = ref('0')
const mouseY = ref('0')
const mouseSpeed = ref([50])
const mouseSmooth = ref(true)

const clickButton = ref('left')
const clickType = ref('single')
const clickPosition = ref('current')
const clickRandomOffset = ref(false)

const typeText = ref('')
const typeVariables = ref(false)
const typeSpeed = ref([50])
const typeHuman = ref(true)
const typeClearInput = ref(false)

const waitType = ref('fixed')
const waitDuration = ref('1000')

const conditionType = ref('image_found')

const loopType = ref('count')
const loopCount = ref('5')
const loopInterval = ref('0')
</script>

<template>
  <!-- 空状态：未选择节点 -->
  <div v-if="!node" class="flex h-full items-center justify-center text-sm text-muted-foreground">
    <div class="text-center">
      <DynamicIcon name="search" :size="32" class="mx-auto text-muted-foreground/50" />
      <p class="mt-2">选择节点以编辑</p>
    </div>
  </div>

  <!-- 已选择节点时的配置界面 -->
  <div v-else class="flex h-full flex-col">
    <!-- 节点头部：类型图标 + 删除/复制按钮 -->
    <div class="flex items-center justify-between border-b border-border px-4 py-3 shrink-0">
      <div class="flex items-center gap-2">
        <DynamicIcon :name="NODE_TYPE_CONFIG[node.type].icon" :size="16" :class="NODE_TYPE_CONFIG[node.type].color" />
        <span class="text-sm font-medium">{{ NODE_TYPE_CONFIG[node.type].label }}</span>
      </div>
      <div class="flex items-center gap-1">
        <Button variant="ghost" size="icon" class="h-7 w-7" @click="emit('duplicate')">
          <DynamicIcon name="copy" :size="14" />
        </Button>
        <Button variant="ghost" size="icon" class="h-7 w-7 text-muted-foreground hover:text-destructive" @click="emit('delete')">
          <DynamicIcon name="trash-2" :size="14" />
        </Button>
      </div>
    </div>

    <!-- 配置表单内容（可滚动） -->
    <div class="flex-1 overflow-auto p-4 space-y-1">
      <!-- ====== 通用：节点名称 ====== -->
      <div class="space-y-3 pb-4">
        <div class="space-y-2">
          <label class="text-xs font-medium">节点名称</label>
          <Input :modelValue="node.label" @update:modelValue="(v: string) => emit('update', { label: v })" class="h-8 text-xs" />
        </div>
      </div>

      <Separator />

      <!-- ======================================================
           类型特定配置（每个节点类型对应不同的配置表单）
           ====================================================== -->

      <div class="pt-2 space-y-1">

        <!-- ====== 查找图片节点 ====== -->
        <template v-if="node.type === 'find'">
          <Collapsible :defaultOpen="true">
            <CollapsibleTrigger class="flex w-full items-center justify-between py-2 text-sm font-medium text-muted-foreground hover:text-foreground">
              目标图片
              <DynamicIcon name="chevron-down" :size="16" />
            </CollapsibleTrigger>
            <CollapsibleContent>
              <div class="space-y-3">
                <div class="flex h-24 items-center justify-center rounded-lg border-2 border-dashed border-border bg-secondary/50 cursor-pointer hover:border-muted-foreground/50 transition-colors">
                  <div class="text-center">
                    <DynamicIcon name="image" :size="24" class="mx-auto text-muted-foreground" />
                    <span class="mt-1 text-xs text-muted-foreground">点击选择图片</span>
                  </div>
                </div>
                <Button variant="outline" size="sm" class="w-full">
                  <DynamicIcon name="plus" :size="12" class="mr-2" />
                  从屏幕截取
                </Button>
              </div>
            </CollapsibleContent>
          </Collapsible>

          <Separator />

          <Collapsible :defaultOpen="true">
            <CollapsibleTrigger class="flex w-full items-center justify-between py-2 text-sm font-medium text-muted-foreground hover:text-foreground">
              匹配设置
              <DynamicIcon name="chevron-down" :size="16" />
            </CollapsibleTrigger>
            <CollapsibleContent>
              <div class="space-y-4 pb-4">
                <div class="space-y-2">
                  <div class="flex items-center justify-between">
                    <label class="text-xs">匹配精度</label>
                    <span class="text-xs text-muted-foreground">{{ findMatchPrecision[0] }}%</span>
                  </div>
                  <Slider v-model="findMatchPrecision" :min="50" :max="100" :step="1" />
                </div>
                <div class="space-y-2">
                  <label class="text-xs">搜索区域</label>
                  <Select v-model="findSearchArea" :options="[{ value: 'full', label: '全屏' }, { value: 'region', label: '指定区域' }, { value: 'window', label: '活动窗口' }]" />
                </div>
                <div class="flex items-center justify-between">
                  <label class="text-xs">灰度匹配</label>
                  <Switch v-model="findGrayscale" />
                </div>
                <div class="space-y-2">
                  <label class="text-xs">超时时间 (秒)</label>
                  <Input v-model="findTimeout" class="h-8 text-xs" />
                </div>
              </div>
            </CollapsibleContent>
          </Collapsible>

          <Separator />

          <Collapsible :defaultOpen="true">
            <CollapsibleTrigger class="flex w-full items-center justify-between py-2 text-sm font-medium text-muted-foreground hover:text-foreground">
              失败处理
              <DynamicIcon name="chevron-down" :size="16" />
            </CollapsibleTrigger>
            <CollapsibleContent>
              <div class="space-y-3 pb-4">
                <Select v-model="findFailAction" :options="[{ value: 'stop', label: '停止执行' }, { value: 'continue', label: '继续执行' }, { value: 'retry', label: '重试' }, { value: 'branch', label: '执行分支' }]" />
              </div>
            </CollapsibleContent>
          </Collapsible>
        </template>

        <!-- ====== 移动鼠标节点 ====== -->
        <template v-if="node.type === 'mouse'">
          <Collapsible :defaultOpen="true">
            <CollapsibleTrigger class="flex w-full items-center justify-between py-2 text-sm font-medium text-muted-foreground hover:text-foreground">
              目标位置
              <DynamicIcon name="chevron-down" :size="16" />
            </CollapsibleTrigger>
            <CollapsibleContent>
              <div class="space-y-3 pb-4">
                <div class="space-y-2">
                  <label class="text-xs">定位方式</label>
                  <Select v-model="mouseMode" :options="[{ value: 'absolute', label: '绝对坐标' }, { value: 'relative', label: '相对偏移' }, { value: 'image', label: '图片中心' }]" />
                </div>
                <div class="grid grid-cols-2 gap-2">
                  <div class="space-y-1">
                    <label class="text-xs">X 坐标</label>
                    <Input v-model="mouseX" class="h-8 text-xs font-mono" />
                  </div>
                  <div class="space-y-1">
                    <label class="text-xs">Y 坐标</label>
                    <Input v-model="mouseY" class="h-8 text-xs font-mono" />
                  </div>
                </div>
                <Button variant="outline" size="sm" class="w-full">
                  <DynamicIcon name="mouse-pointer" :size="12" class="mr-2" />
                  从屏幕选取
                </Button>
              </div>
            </CollapsibleContent>
          </Collapsible>

          <Separator />

          <Collapsible :defaultOpen="true">
            <CollapsibleTrigger class="flex w-full items-center justify-between py-2 text-sm font-medium text-muted-foreground hover:text-foreground">
              移动选项
              <DynamicIcon name="chevron-down" :size="16" />
            </CollapsibleTrigger>
            <CollapsibleContent>
              <div class="space-y-4 pb-4">
                <div class="space-y-2">
                  <div class="flex items-center justify-between">
                    <label class="text-xs">移动速度</label>
                    <span class="text-xs text-muted-foreground">中速</span>
                  </div>
                  <Slider v-model="mouseSpeed" :min="0" :max="100" :step="1" />
                </div>
                <div class="flex items-center justify-between">
                  <label class="text-xs">平滑移动</label>
                  <Switch v-model="mouseSmooth" />
                </div>
              </div>
            </CollapsibleContent>
          </Collapsible>
        </template>

        <!-- ====== 点击节点 ====== -->
        <template v-if="node.type === 'click'">
          <Collapsible :defaultOpen="true">
            <CollapsibleTrigger class="flex w-full items-center justify-between py-2 text-sm font-medium text-muted-foreground hover:text-foreground">
              点击设置
              <DynamicIcon name="chevron-down" :size="16" />
            </CollapsibleTrigger>
            <CollapsibleContent>
              <div class="space-y-3 pb-4">
                <div class="space-y-2">
                  <label class="text-xs">按钮</label>
                  <Select v-model="clickButton" :options="[{ value: 'left', label: '左键' }, { value: 'right', label: '右键' }, { value: 'middle', label: '中键' }]" />
                </div>
                <div class="space-y-2">
                  <label class="text-xs">点击类型</label>
                  <Select v-model="clickType" :options="[{ value: 'single', label: '单击' }, { value: 'double', label: '双击' }, { value: 'triple', label: '三击' }, { value: 'hold', label: '按住' }]" />
                </div>
              </div>
            </CollapsibleContent>
          </Collapsible>

          <Separator />

          <Collapsible :defaultOpen="true">
            <CollapsibleTrigger class="flex w-full items-center justify-between py-2 text-sm font-medium text-muted-foreground hover:text-foreground">
              位置
              <DynamicIcon name="chevron-down" :size="16" />
            </CollapsibleTrigger>
            <CollapsibleContent>
              <div class="space-y-3 pb-4">
                <div class="space-y-2">
                  <label class="text-xs">点击位置</label>
                  <Select v-model="clickPosition" :options="[{ value: 'current', label: '当前位置' }, { value: 'image', label: '图片位置' }, { value: 'absolute', label: '指定坐标' }]" />
                </div>
                <div class="flex items-center justify-between">
                  <label class="text-xs">随机偏移</label>
                  <Switch v-model="clickRandomOffset" />
                </div>
              </div>
            </CollapsibleContent>
          </Collapsible>
        </template>

        <!-- ====== 输入文本节点 ====== -->
        <template v-if="node.type === 'type'">
          <Collapsible :defaultOpen="true">
            <CollapsibleTrigger class="flex w-full items-center justify-between py-2 text-sm font-medium text-muted-foreground hover:text-foreground">
              输入内容
              <DynamicIcon name="chevron-down" :size="16" />
            </CollapsibleTrigger>
            <CollapsibleContent>
              <div class="space-y-3 pb-4">
                <div class="space-y-2">
                  <label class="text-xs">文本内容</label>
                  <Textarea v-model="typeText" placeholder="输入要键入的文本..." class="min-h-[80px] text-xs" />
                </div>
                <div class="flex items-center justify-between">
                  <label class="text-xs">支持变量</label>
                  <Switch v-model="typeVariables" />
                </div>
              </div>
            </CollapsibleContent>
          </Collapsible>

          <Separator />

          <Collapsible :defaultOpen="true">
            <CollapsibleTrigger class="flex w-full items-center justify-between py-2 text-sm font-medium text-muted-foreground hover:text-foreground">
              输入选项
              <DynamicIcon name="chevron-down" :size="16" />
            </CollapsibleTrigger>
            <CollapsibleContent>
              <div class="space-y-4 pb-4">
                <div class="space-y-2">
                  <div class="flex items-center justify-between">
                    <label class="text-xs">输入速度</label>
                    <span class="text-xs text-muted-foreground">每字 50ms</span>
                  </div>
                  <Slider v-model="typeSpeed" :min="0" :max="200" :step="10" />
                </div>
                <div class="flex items-center justify-between">
                  <label class="text-xs">模拟真人输入</label>
                  <Switch v-model="typeHuman" />
                </div>
                <div class="flex items-center justify-between">
                  <label class="text-xs">自动清空输入框</label>
                  <Switch v-model="typeClearInput" />
                </div>
              </div>
            </CollapsibleContent>
          </Collapsible>
        </template>

        <!-- ====== 等待节点 ====== -->
        <template v-if="node.type === 'wait'">
          <Collapsible :defaultOpen="true">
            <CollapsibleTrigger class="flex w-full items-center justify-between py-2 text-sm font-medium text-muted-foreground hover:text-foreground">
              等待设置
              <DynamicIcon name="chevron-down" :size="16" />
            </CollapsibleTrigger>
            <CollapsibleContent>
              <div class="space-y-3 pb-4">
                <div class="space-y-2">
                  <label class="text-xs">等待类型</label>
                  <Select v-model="waitType" :options="[{ value: 'fixed', label: '固定时间' }, { value: 'random', label: '随机时间' }, { value: 'image', label: '等待图片出现' }, { value: 'disappear', label: '等待图片消失' }]" />
                </div>
                <div class="space-y-2">
                  <label class="text-xs">等待时间 (毫秒)</label>
                  <Input v-model="waitDuration" class="h-8 text-xs font-mono" />
                </div>
              </div>
            </CollapsibleContent>
          </Collapsible>
        </template>

        <!-- ====== 条件分支节点 ====== -->
        <template v-if="node.type === 'condition'">
          <Collapsible :defaultOpen="true">
            <CollapsibleTrigger class="flex w-full items-center justify-between py-2 text-sm font-medium text-muted-foreground hover:text-foreground">
              条件设置
              <DynamicIcon name="chevron-down" :size="16" />
            </CollapsibleTrigger>
            <CollapsibleContent>
              <div class="space-y-3 pb-4">
                <div class="space-y-2">
                  <label class="text-xs">条件类型</label>
                  <Select v-model="conditionType" :options="[{ value: 'image_found', label: '图片已找到' }, { value: 'image_not_found', label: '图片未找到' }, { value: 'variable', label: '变量比较' }, { value: 'counter', label: '计数器' }]" />
                </div>
                <div class="p-3 rounded-lg bg-secondary/50 text-xs text-muted-foreground">
                  <p>如果条件为真，执行 "是" 分支</p>
                  <p class="mt-1">否则执行 "否" 分支</p>
                </div>
              </div>
            </CollapsibleContent>
          </Collapsible>
        </template>

        <!-- ====== 循环节点 ====== -->
        <template v-if="node.type === 'loop'">
          <Collapsible :defaultOpen="true">
            <CollapsibleTrigger class="flex w-full items-center justify-between py-2 text-sm font-medium text-muted-foreground hover:text-foreground">
              循环设置
              <DynamicIcon name="chevron-down" :size="16" />
            </CollapsibleTrigger>
            <CollapsibleContent>
              <div class="space-y-3 pb-4">
                <div class="space-y-2">
                  <label class="text-xs">循环类型</label>
                  <Select v-model="loopType" :options="[{ value: 'count', label: '固定次数' }, { value: 'while', label: '条件循环' }, { value: 'infinite', label: '无限循环' }]" />
                </div>
                <div class="space-y-2">
                  <label class="text-xs">循环次数</label>
                  <Input v-model="loopCount" class="h-8 text-xs font-mono" />
                </div>
                <div class="space-y-2">
                  <label class="text-xs">循环间隔 (毫秒)</label>
                  <Input v-model="loopInterval" class="h-8 text-xs font-mono" />
                </div>
              </div>
            </CollapsibleContent>
          </Collapsible>
        </template>
      </div>
    </div>
  </div>
</template>
