<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  label: string
  options: string[]
  selectedValues: string[]
  allSelected: boolean
  partiallySelected: boolean
  onToggleAll: (checked: boolean) => void
  onToggleValue: (value: string) => void
}>()

const allSelectedModel = computed({
  get: () => props.allSelected,
  set: (value: boolean) => props.onToggleAll(value),
})
</script>

<template>
  <div class="filter-field">
    <div class="filter-dropdown filter-label-trigger">
      <span class="filter-label">{{ props.label }}</span>
      <div class="filter-menu">
        <label class="filter-option">
          <input
            type="checkbox"
            v-model="allSelectedModel"
            :indeterminate.prop="props.partiallySelected"
          />
          <span>全部</span>
        </label>
        <label v-for="item in props.options" :key="item" class="filter-option">
          <input
            type="checkbox"
            :checked="props.selectedValues.includes(item)"
            @change="props.onToggleValue(item)"
          />
          <span>{{ item }}</span>
        </label>
      </div>
    </div>
  </div>
</template>
