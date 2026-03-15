<script setup lang="ts">
import { computed } from 'vue'

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
    <button @click="props.onFirst">&lt;&lt;</button>
    <button @click="props.onPrev">&lt;</button>
    <button
      v-for="page in props.visiblePages"
      :key="page"
      :class="{ active: page === props.currentPage }"
      @click="props.onGo(page)"
    >
      {{ page + 1 }}
    </button>
    <button @click="props.onNext">&gt;</button>
    <button @click="props.onLast">&gt;&gt;</button>
  </div>
</template>
