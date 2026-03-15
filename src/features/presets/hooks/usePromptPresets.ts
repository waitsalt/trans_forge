import { computed, ref } from 'vue'
import { presetBridge } from '../api/presetBridge'
import { toErrorMessage, toPositiveInt } from '../../../shared/utils/core'
import type { PromptPreset } from '../types'
import { normalizePromptPreset } from '../utils'

interface ToastLike {
  success: (message: string) => void
  error: (message: string) => void
}

export function usePromptPresets(toast: ToastLike) {
  const promptPresetLibrary = ref<PromptPreset[]>([])
  const promptPresetAll = ref<PromptPreset[]>([])
  const promptPresetSearchKeyword = ref<string>('')
  const selectedPromptPresetSet = ref<Set<string>>(new Set())
  const showPromptPresetEditor = ref<boolean>(false)
  const editingPromptPresetName = ref<string | null>(null)
  const promptPresetPageSize = ref<number>(10)
  const promptPresetPage = ref<number>(0)
  const promptPresetTotalPages = ref<number>(0)
  const draftPromptPresetName = ref<string>('')
  const draftPromptPresetLanguage = ref<string>('ZH')
  const draftPromptPresetPrompt = ref<string>('')

  const normalizedPromptPresetPageSize = computed(() => {
    return toPositiveInt(Number(promptPresetPageSize.value), 10)
  })

  function buildDraftPromptPreset(): PromptPreset | null {
    const name = draftPromptPresetName.value.trim()
    const prompt = draftPromptPresetPrompt.value.trim()
    if (!name) {
      toast.error('提示词名称不能为空')
      return null
    }
    if (!prompt) {
      toast.error('提示词内容不能为空')
      return null
    }
    return {
      name,
      language: draftPromptPresetLanguage.value,
      prompt,
    }
  }

  async function loadPromptPresets() {
    const keyword = promptPresetSearchKeyword.value.trim()
    const response = await presetBridge.queryPromptPresets({
      keyword: keyword.length > 0 ? keyword : null,
      page: promptPresetPage.value,
      pageSize: normalizedPromptPresetPageSize.value,
    })
    const normalized = (response.items ?? []).map((item, idx) =>
      normalizePromptPreset(item, `preset-${idx + 1}`),
    )
    promptPresetLibrary.value = normalized
    promptPresetTotalPages.value = response.total_pages ?? 0
    promptPresetPage.value = response.page ?? 0
  }

  async function refreshPromptPresetOptions() {
    const presets = await presetBridge.listPromptPresets()
    promptPresetAll.value = (presets ?? []).map((item, idx) =>
      normalizePromptPreset(item, `preset-${idx + 1}`),
    )
  }

  function openCreatePromptPreset() {
    showPromptPresetEditor.value = true
    editingPromptPresetName.value = null
    draftPromptPresetName.value = ''
    draftPromptPresetLanguage.value = 'ZH'
    draftPromptPresetPrompt.value = ''
  }

  async function openEditPromptPreset(name: string) {
    showPromptPresetEditor.value = true
    editingPromptPresetName.value = name
    try {
      const loaded = await presetBridge.getPromptPreset({ name })
      const preset = normalizePromptPreset(loaded, name)
      draftPromptPresetName.value = preset.name
      draftPromptPresetLanguage.value = preset.language
      draftPromptPresetPrompt.value = preset.prompt
    } catch (error) {
      toast.error(`加载提示词失败: ${toErrorMessage(error)}`)
    }
  }

  function cancelEditPromptPreset() {
    showPromptPresetEditor.value = false
    editingPromptPresetName.value = null
  }

  async function savePromptPreset() {
    const preset = buildDraftPromptPreset()
    if (!preset) {
      return
    }
    try {
      if (editingPromptPresetName.value) {
        const originalName = editingPromptPresetName.value
        await presetBridge.updatePromptPreset({ originalName, preset })
        if (selectedPromptPresetSet.value.delete(originalName)) {
          selectedPromptPresetSet.value.add(preset.name)
        }
        toast.success(`已更新提示词: ${preset.name}`)
      } else {
        await presetBridge.createPromptPreset({ preset })
        toast.success(`已新增提示词: ${preset.name}`)
      }
      await loadPromptPresets()
      await refreshPromptPresetOptions()
      showPromptPresetEditor.value = false
      editingPromptPresetName.value = null
    } catch (error) {
      toast.error(`保存提示词失败: ${toErrorMessage(error)}`)
    }
  }

  async function deletePromptPreset(name: string) {
    if (!name.trim()) {
      return
    }
    try {
      await presetBridge.deletePromptPreset({ name })
      selectedPromptPresetSet.value.delete(name)
      await loadPromptPresets()
      await refreshPromptPresetOptions()
      toast.success(`已删除提示词: ${name}`)
    } catch (error) {
      toast.error(`删除提示词失败: ${toErrorMessage(error)}`)
    }
  }

  function toggleSelectPromptPreset(name: string) {
    const next = new Set(selectedPromptPresetSet.value)
    if (next.has(name)) {
      next.delete(name)
    } else {
      next.add(name)
    }
    selectedPromptPresetSet.value = next
  }

  function selectAllVisiblePromptPresets() {
    const next = new Set(selectedPromptPresetSet.value)
    for (const preset of promptPresetLibrary.value) {
      next.add(preset.name)
    }
    selectedPromptPresetSet.value = next
  }

  function updatePromptPresetPageSize(value: number) {
    promptPresetPageSize.value = toPositiveInt(value, 10)
  }

  function updatePromptPresetPage(value: number) {
    const numeric = Number.isFinite(value) ? Math.floor(value) : 0
    const maxPage = Math.max(promptPresetTotalPages.value - 1, 0)
    promptPresetPage.value = Math.min(Math.max(numeric, 0), maxPage)
  }

  function goToFirstPromptPresetPage() {
    promptPresetPage.value = 0
  }

  function goToPrevPromptPresetPage() {
    promptPresetPage.value = Math.max(promptPresetPage.value - 1, 0)
  }

  function goToLastPromptPresetPage() {
    if (promptPresetTotalPages.value > 0) {
      promptPresetPage.value = promptPresetTotalPages.value - 1
    }
  }

  function goToNextPromptPresetPage() {
    const maxPage = Math.max(promptPresetTotalPages.value - 1, 0)
    promptPresetPage.value = Math.min(promptPresetPage.value + 1, maxPage)
  }

  function goToPromptPresetPage(page: number) {
    const maxPage = Math.max(promptPresetTotalPages.value - 1, 0)
    promptPresetPage.value = Math.min(Math.max(Math.floor(page), 0), maxPage)
  }

  return {
    promptPresetLibrary,
    promptPresetAll,
    promptPresetSearchKeyword,
    selectedPromptPresetSet,
    showPromptPresetEditor,
    editingPromptPresetName,
    promptPresetPageSize,
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
  }
}
