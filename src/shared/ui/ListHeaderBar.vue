<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  title: string
  searchKeyword: string
  searchPlaceholder: string
  addLabel: string
  selectedCount: number
  onUpdateSearchKeyword: (value: string) => void
  onAdd: () => void
  onSelectAll: () => void
  onBulkDelete: () => void
}>()

const searchKeywordModel = computed({
  get: () => props.searchKeyword,
  set: (value: string) => props.onUpdateSearchKeyword(value),
})
</script>

<template>
  <div class="section-header">
    <h2>{{ props.title }}</h2>
    <div class="header-actions">
      <input
        class="search-input"
        type="text"
        v-model="searchKeywordModel"
        :placeholder="props.searchPlaceholder"
      />
      <button class="btn-primary" @click="props.onAdd">{{ props.addLabel }}</button>
      <button @click="props.onSelectAll">全选</button>
      <button v-if="props.selectedCount > 0" class="danger" @click="props.onBulkDelete">批量删除</button>
    </div>
  </div>
</template>
