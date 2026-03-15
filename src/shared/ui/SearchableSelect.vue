<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'

interface SelectOption {
  value: string
  label: string
}

const props = withDefaults(
  defineProps<{
    options: SelectOption[]
    placeholder?: string
    allowCustom?: boolean
    emptyText?: string
    editable?: boolean
    filterable?: boolean
    dropdownEnabled?: boolean
    onOpen?: () => void
    onQuery?: (value: string) => void
  }>(),
  {
    placeholder: '',
    allowCustom: false,
    emptyText: '没有匹配项',
    editable: true,
    filterable: true,
    dropdownEnabled: true,
  },
)

const modelValue = defineModel<string>({ required: true })

const rootRef = ref<HTMLElement | null>(null)
const inputRef = ref<HTMLInputElement | null>(null)
const open = ref<boolean>(false)
const keyword = ref<string>('')
const highlightedIndex = ref<number>(-1)

const selectedOption = computed(() => props.options.find((item) => item.value === modelValue.value) ?? null)
const filteredOptions = computed(() => {
  if (!props.filterable) {
    return props.options
  }
  const needle = keyword.value.trim().toLowerCase()
  if (!needle) {
    return props.options
  }
  return props.options.filter(
    (item) => item.label.toLowerCase().includes(needle) || item.value.toLowerCase().includes(needle),
  )
})

function syncKeywordFromValue() {
  if (props.allowCustom) {
    keyword.value = modelValue.value
  } else {
    keyword.value = selectedOption.value?.label ?? ''
  }
}

watch(
  () => modelValue.value,
  () => {
    if (!open.value || props.allowCustom) {
      syncKeywordFromValue()
    }
  },
  { immediate: true },
)

function openDropdown() {
  if (!props.dropdownEnabled) {
    return
  }
  open.value = true
  props.onOpen?.()
  if (filteredOptions.value.length > 0) {
    highlightedIndex.value = Math.max(
      filteredOptions.value.findIndex((item) => item.value === modelValue.value),
      0,
    )
  }
}

function onInput(value: string) {
  if (!props.editable) {
    return
  }
  keyword.value = value
  props.onQuery?.(value)
  if (props.dropdownEnabled) {
    open.value = true
  }
  if (props.allowCustom) {
    modelValue.value = value
  }
}

function selectOption(item: SelectOption) {
  modelValue.value = item.value
  keyword.value = item.label
  open.value = false
  highlightedIndex.value = -1
}

function closeDropdown() {
  if (!props.allowCustom) {
    syncKeywordFromValue()
  }
  open.value = false
  highlightedIndex.value = -1
}

function handleDocumentPointerDown(event: MouseEvent) {
  const target = event.target as Node | null
  if (!target || !rootRef.value) {
    return
  }
  if (!rootRef.value.contains(target)) {
    closeDropdown()
  }
}

function moveHighlight(step: number) {
  if (!open.value) {
    openDropdown()
    return
  }
  const size = filteredOptions.value.length
  if (size === 0) {
    highlightedIndex.value = -1
    return
  }
  if (highlightedIndex.value < 0) {
    highlightedIndex.value = step > 0 ? 0 : size - 1
  } else {
    highlightedIndex.value = (highlightedIndex.value + step + size) % size
  }
  scrollHighlightedIntoView()
}

function confirmHighlight() {
  if (!open.value) {
    openDropdown()
    return
  }
  const item = filteredOptions.value[highlightedIndex.value]
  if (item) {
    selectOption(item)
  }
}

function onKeydown(event: KeyboardEvent) {
  if (event.key === 'ArrowDown') {
    event.preventDefault()
    event.stopPropagation()
    moveHighlight(1)
    return
  }
  if (event.key === 'ArrowUp') {
    event.preventDefault()
    event.stopPropagation()
    moveHighlight(-1)
    return
  }
  if (event.key === 'Enter') {
    event.preventDefault()
    event.stopPropagation()
    confirmHighlight()
  }
}

function scrollHighlightedIntoView() {
  nextTick(() => {
    const root = rootRef.value
    if (!root || highlightedIndex.value < 0) {
      return
    }
    const target = root.querySelector<HTMLElement>(`[data-option-index="${highlightedIndex.value}"]`)
    target?.scrollIntoView({ block: 'nearest' })
  })
}

watch(filteredOptions, (items) => {
  if (items.length === 0) {
    highlightedIndex.value = -1
    return
  }
  if (highlightedIndex.value < 0 || highlightedIndex.value >= items.length) {
    highlightedIndex.value = Math.max(
      items.findIndex((item) => item.value === modelValue.value),
      0,
    )
  }
})

document.addEventListener('mousedown', handleDocumentPointerDown)
onBeforeUnmount(() => {
  document.removeEventListener('mousedown', handleDocumentPointerDown)
})
</script>

<template>
  <div ref="rootRef" class="searchable-select">
    <input
      ref="inputRef"
      type="text"
      v-model="keyword"
      :placeholder="placeholder"
      :readonly="!editable"
      @focus="openDropdown"
      @click="openDropdown"
      @input="onInput(keyword)"
      @keydown="onKeydown"
    />
    <div v-if="open" class="searchable-dropdown">
      <button
        v-for="(item, index) in filteredOptions"
        :key="item.value"
        type="button"
        class="searchable-option"
        :class="{ highlighted: index === highlightedIndex }"
        :data-option-index="index"
        @click="selectOption(item)"
        @mouseenter="highlightedIndex = index"
      >
        {{ item.label }}
      </button>
      <div v-if="filteredOptions.length === 0" class="searchable-tip">{{ emptyText }}</div>
    </div>
  </div>
</template>

<style scoped>
.searchable-select {
  position: relative;
}

.searchable-select input[readonly] {
  cursor: pointer;
}

.searchable-dropdown {
  position: absolute;
  left: 0;
  right: 0;
  top: calc(100% + 6px);
  max-height: 220px;
  overflow-y: auto;
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--bg-elev);
  box-shadow: 0 12px 28px -20px rgba(0, 0, 0, 0.6);
  z-index: 80;
}

.searchable-option {
  width: 100%;
  height: 36px;
  border: 0;
  border-bottom: 1px solid color-mix(in srgb, var(--border) 75%, transparent);
  text-align: left;
  padding: 0 11px;
  background: transparent;
  color: var(--text);
  cursor: pointer;
}

.searchable-option:last-of-type {
  border-bottom: 0;
}

.searchable-option.highlighted {
  background: var(--bg-muted);
}

.searchable-option:hover {
  background: var(--bg-muted);
}

.searchable-tip {
  padding: 10px 11px;
  color: var(--text-dim);
  font-size: 13px;
}
</style>
