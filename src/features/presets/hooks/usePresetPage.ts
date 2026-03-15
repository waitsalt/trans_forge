import { computed, ref, watch } from 'vue'
import { toast } from 'vue-sonner'
import { defaultLanguages } from '../../../shared/constants'
import { presetBridge } from '../api/presetBridge'
import { toErrorMessage } from '../../../shared/utils/core'
import { usePromptPresets } from './usePromptPresets'
import { usePageDataSync } from '../../../shared/composables/usePageDataSync'
import { useVisiblePages } from '../../../shared/composables/useVisiblePages'

export function usePresetPage() {
  const languages = ref([...defaultLanguages])

  const {
    promptPresetLibrary,
    promptPresetSearchKeyword,
    selectedPromptPresetSet,
    showPromptPresetEditor,
    editingPromptPresetName,
    promptPresetPage,
    promptPresetTotalPages,
    draftPromptPresetName,
    draftPromptPresetLanguage,
    draftPromptPresetPrompt,
    normalizedPromptPresetPageSize,
    loadPromptPresets,
    refreshPromptPresetOptions,
    openCreatePromptPreset,
    openEditPromptPreset,
    cancelEditPromptPreset,
    savePromptPreset,
    deletePromptPreset,
    toggleSelectPromptPreset,
    selectAllVisiblePromptPresets,
    updatePromptPresetPageSize,
    updatePromptPresetPage,
    goToFirstPromptPresetPage,
    goToPrevPromptPresetPage,
    goToLastPromptPresetPage,
    goToNextPromptPresetPage,
    goToPromptPresetPage,
  } = usePromptPresets(toast)

  usePageDataSync({
    toast,
    initialTasks: [
      { task: loadPromptPresets, errorPrefix: '加载提示词失败' },
      { task: refreshPromptPresetOptions, errorPrefix: '加载提示词选项失败' },
    ],
    watchTasks: [
      {
        filterSources: [promptPresetSearchKeyword, normalizedPromptPresetPageSize],
        page: promptPresetPage,
        task: loadPromptPresets,
        errorPrefix: '加载提示词失败',
      },
    ],
  })

  const selectedCount = computed(() => selectedPromptPresetSet.value.size)
  const totalPages = computed(() => promptPresetTotalPages.value)

  const visiblePages = useVisiblePages(promptPresetPage, totalPages)

  const presets = computed(() =>
    promptPresetLibrary.value.map((preset, idx) => ({
      serial: promptPresetPage.value * normalizedPromptPresetPageSize.value + idx + 1,
      name: preset.name,
      language: languages.value.find((item) => item.code === preset.language)?.name ?? preset.language,
      prompt: preset.prompt,
      selected: selectedPromptPresetSet.value.has(preset.name),
    })),
  )

  watch(totalPages, () => {
    const maxPage = Math.max(totalPages.value - 1, 0)
    if (promptPresetPage.value > maxPage) {
      updatePromptPresetPage(maxPage)
    }
  })

  async function savePreset() {
    await savePromptPreset()
  }

  async function deletePreset(name: string) {
    await deletePromptPreset(name)
  }

  async function bulkDeleteSelected() {
    const names = Array.from(selectedPromptPresetSet.value)
    if (names.length === 0) {
      return
    }
    const ok = window.confirm(`确认删除选中的 ${names.length} 个提示词吗？`)
    if (!ok) {
      return
    }
    try {
      const removed = await presetBridge.deletePromptPresets({ names })
      selectedPromptPresetSet.value = new Set(
        Array.from(selectedPromptPresetSet.value).filter((item) => !names.includes(item)),
      )
      await loadPromptPresets()
      await refreshPromptPresetOptions()
      toast.success(`已批量删除 ${removed} 个提示词`)
    } catch (error) {
      toast.error(`批量删除提示词失败: ${toErrorMessage(error)}`)
    }
  }

  return {
    languages,
    promptPresetSearchKeyword,
    selectedCount,
    showPromptPresetEditor,
    openCreatePromptPreset,
    selectAllVisiblePromptPresets,
    bulkDeleteSelected,
    normalizedPromptPresetPageSize,
    updatePromptPresetPageSize,
    promptPresetPage,
    updatePromptPresetPage,
    totalPages,
    goToFirstPromptPresetPage,
    goToPrevPromptPresetPage,
    visiblePages,
    goToPromptPresetPage,
    goToNextPromptPresetPage,
    goToLastPromptPresetPage,
    presets,
    toggleSelectPromptPreset,
    openEditPromptPreset,
    deletePreset,
    editingPromptPresetName,
    draftPromptPresetName,
    draftPromptPresetLanguage,
    draftPromptPresetPrompt,
    savePreset,
    cancelEditPromptPreset,
  }
}
