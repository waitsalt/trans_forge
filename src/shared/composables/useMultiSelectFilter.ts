import { computed, ref, watch, type Ref } from 'vue'

type UseMultiSelectFilterOptions = {
  initializeAll?: boolean
}

export function useMultiSelectFilter(
  availableValues: Ref<string[]>,
  options: UseMultiSelectFilterOptions = {},
) {
  const { initializeAll = true } = options
  const selectedValues = ref<string[]>([])
  const initialized = ref(false)

  watch(
    availableValues,
    (available) => {
      if (!initialized.value && initializeAll) {
        selectedValues.value = [...available]
        initialized.value = true
        return
      }
      selectedValues.value = selectedValues.value.filter((value) => available.includes(value))
    },
    { immediate: true },
  )

  const allSelected = computed(() => {
    const available = availableValues.value
    return available.length > 0 && available.every((value) => selectedValues.value.includes(value))
  })

  const partiallySelected = computed(() => selectedValues.value.length > 0 && !allSelected.value)

  function toggleValue(value: string) {
    const next = new Set(selectedValues.value)
    if (next.has(value)) {
      next.delete(value)
    } else {
      next.add(value)
    }
    selectedValues.value = Array.from(next.values())
  }

  function toggleAll(checked: boolean) {
    selectedValues.value = checked ? [...availableValues.value] : []
  }

  return {
    selectedValues,
    allSelected,
    partiallySelected,
    toggleValue,
    toggleAll,
  }
}
