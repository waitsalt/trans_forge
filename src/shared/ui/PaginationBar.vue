<script setup lang="ts">
import { computed } from 'vue'
import { EpArrowLeft, EpArrowRight, EpDArrowLeft, EpDArrowRight } from 'vue-icons-plus/ep'

const props = defineProps<{
  pageSize: number
  currentPage: number
  totalPages: number
  visiblePages: number[]
  onUpdatePageSize: (value: number) => void
  onUpdateCurrentPage: (value: number) => void
  onFirst: () => void
  onPrev: () => void
  onNext: () => void
  onLast: () => void
  onGo: (value: number) => void
}>()

const pageSizeModel = computed({
  get: () => props.pageSize,
  set: (value: number) => props.onUpdatePageSize(Number(value)),
})

const currentPageModel = computed({
  get: () => props.currentPage + 1,
  set: (value: number) => props.onUpdateCurrentPage(Number(value) - 1),
})

const disablePrev = computed(() => props.totalPages <= 0 || props.currentPage <= 0)
const disableNext = computed(() => props.totalPages <= 0 || props.currentPage >= props.totalPages - 1)
</script>

<template>
  <div class="pagination-wrap">
    <input
      type="number"
      min="1"
      v-model.number="pageSizeModel"
    />
    <span>条/页</span>
    <span>第</span>
    <input
      type="number"
      min="1"
      v-model.number="currentPageModel"
    />
    <span> / {{ props.totalPages }}</span>
    <button class="btn btn--ghost btn--icon" aria-label="最前" :disabled="disablePrev" @click="props.onFirst">
      <EpDArrowLeft class="inline-icon" />
    </button>
    <button class="btn btn--ghost btn--icon" aria-label="上一个" :disabled="disablePrev" @click="props.onPrev">
      <EpArrowLeft class="inline-icon" />
    </button>
    <button
      v-for="page in props.visiblePages"
      :key="page"
      :class="{ active: page === props.currentPage }"
      @click="props.onGo(page)"
    >
      {{ page + 1 }}
    </button>
    <button class="btn btn--ghost btn--icon" aria-label="下一个" :disabled="disableNext" @click="props.onNext">
      <EpArrowRight class="inline-icon" />
    </button>
    <button class="btn btn--ghost btn--icon" aria-label="最后" :disabled="disableNext" @click="props.onLast">
      <EpDArrowRight class="inline-icon" />
    </button>
  </div>
</template>
