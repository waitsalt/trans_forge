import { computed, onBeforeUnmount, ref, watch } from 'vue'
import { toast } from 'vue-sonner'
import { defaultLanguages } from '../../../shared/constants'
import type { Language } from '../../../shared/types/ui'
import { toErrorMessage, toPositiveInt } from '../../../shared/utils/core'
import { providerBridge } from '../../providers/api/providerBridge'
import type { Provider } from '../../providers/types'
import { presetBridge } from '../../presets/api/presetBridge'
import type { PromptPreset } from '../../presets/types'
import { projectBridge } from '../api/projectBridge'
import type { ProjectStatusCode } from '../types'
import { useProjectConfigs } from './useProjectConfigs'
import { useProjectRuntimeActions } from './useProjectRuntimeActions'
import { useProjectPathPickers } from './useProjectPathPickers'
import { usePageDataSync } from '../../../shared/composables/usePageDataSync'
import { useVisiblePages } from '../../../shared/composables/useVisiblePages'
import { useMultiSelectFilter } from '../../../shared/composables/useMultiSelectFilter'

type ProjectProgressEntry = {
  status: ProjectStatusCode
  total: number
  processed: number
  error: number
  is_running: boolean
}

export function useProjectPage() {
  const languages = ref<Language[]>([...defaultLanguages])
  const projectProgressMap = ref<Record<string, ProjectProgressEntry>>({})
  const promptPresetAll = ref<PromptPreset[]>([])

  const {
    projectLibrary,
    projectSearchKeyword,
    projectRunStatusFilters,
    selectedProjectSet,
    showProjectEditor,
    editingProjectName,
    projectPage,
    projectTotalPages,
    draftProjectName,
    draftProjectSourceLanguage,
    draftProjectTargetLanguage,
    draftProjectProviderName,
    draftProjectConcurrentLimit,
    draftProjectPrompt,
    draftProjectPromptPresetName,
    draftProjectInputPath,
    draftProjectOutputPath,
    normalizedProjectPageSize,
    loadProjects,
    applyPromptPresetToProject,
    saveProjectProfile,
    openCreateProject: createProjectProfileEditor,
    openEditProject,
    cancelEditProject,
    toggleSelectProject,
    selectAllVisibleProjects,
    deleteProject,
    updateProjectProfilePageSize,
    updateProjectProfilePage,
  } = useProjectConfigs(toast, projectProgressMap, promptPresetAll)

  const {
    showProjectDetail,
    detailProjectName,
    detailFinalPrompt,
    detailItems,
    runningDetailItemIds,
    exportingProjectSet,
    activeProjectRunName,
    isTranslating,
    stopGlobalProgressPolling,
    startOrPauseProject,
    retryProjectErrors,
    retryProjectAll,
    openProjectDetail,
    closeProjectDetail,
    updateDetailItemText: applyDetailItemTextUpdate,
    runDetailItem: runDetailItemAction,
    exportProject,
    clearProjectCache,
  } = useProjectRuntimeActions({
    toast,
    loadProjects,
    projectLibrary,
    projectProgressMap,
  })

  const {
    selectInputFolder,
    selectInputDirectory,
    selectOutputFolder,
  } = useProjectPathPickers({
    draftProjectInputPath,
    draftProjectOutputPath,
  })

  const providerLibrary = ref<Provider[]>([])

  async function loadProviderOptions() {
    const response = await providerBridge.queryProviders({
      keyword: null,
      formatTypes: ['openai', 'google', 'anthropic'],
      page: 0,
      pageSize: 1000,
    })
    providerLibrary.value = response.items ?? []
  }

  async function refreshPromptPresetOptions() {
    const presets = await presetBridge.listPromptPresets()
    promptPresetAll.value = presets ?? []
  }

  usePageDataSync({
    toast,
    initialTasks: [
      { task: loadProjects, errorPrefix: '加载项目配置失败' },
      { task: loadProviderOptions, errorPrefix: '加载 Provider 失败' },
      { task: refreshPromptPresetOptions, errorPrefix: '加载提示词选项失败' },
    ],
    watchTasks: [
      {
        filterSources: [projectSearchKeyword, normalizedProjectPageSize, projectRunStatusFilters],
        page: projectPage,
        task: loadProjects,
        errorPrefix: '加载项目配置失败',
      },
    ],
  })

  onBeforeUnmount(() => {
    stopGlobalProgressPolling()
  })

  const searchKeyword = projectSearchKeyword
  const selectedCount = computed(() => selectedProjectSet.value.size)
  const pageSize = normalizedProjectPageSize
  const currentPage = projectPage
  const totalPages = computed(() => projectTotalPages.value)
  const showEditor = showProjectEditor
  const draftInputPath = draftProjectInputPath
  const draftOutputPath = draftProjectOutputPath
  const draftSourceLanguage = draftProjectSourceLanguage
  const draftTargetLanguage = draftProjectTargetLanguage
  const draftProviderName = draftProjectProviderName
  const draftConcurrentLimit = draftProjectConcurrentLimit
  const draftPrompt = draftProjectPrompt
  const draftPromptPresetName = draftProjectPromptPresetName
  const promptPresetOptions = computed(() =>
    promptPresetAll.value.map((preset) => ({ value: preset.name, label: preset.name })),
  )
  const providerOptions = computed(() =>
    providerLibrary.value.map((item) => ({ value: item.name, label: item.name })),
  )
  const showDetail = showProjectDetail
  const detailPrompt = detailFinalPrompt

  function openCreateProject() {
    createProjectProfileEditor(providerOptions.value[0]?.value ?? '')
  }

  const EMPTY_PROJECT_PROGRESS: ProjectProgressEntry = {
    status: 'not_started',
    total: 0,
    processed: 0,
    error: 0,
    is_running: false,
  }

  function resolveLanguageLabel(languageCode: string) {
    const normalizedCode = languageCode.trim().toUpperCase()
    const matchedLanguage = languages.value.find((language) => language.code.toUpperCase() === normalizedCode)
    return matchedLanguage?.name ?? languageCode
  }

  const projectConfigs = computed(() =>
    projectLibrary.value.map((cfg, idx) => {
      const progress = projectProgressMap.value[cfg.name] ?? EMPTY_PROJECT_PROGRESS
      const statusCode = progress.status ?? cfg.run_status ?? 'not_started'
      const progressPending = Math.max(progress.total - progress.processed - progress.error, 0)
      const runningFromProgress = progress.is_running || progress.status === 'running'
      const runningFromActive = activeProjectRunName.value === cfg.name && isTranslating.value
      const runningCount =
        runningFromProgress || runningFromActive
          ? Math.max(1, Math.min(Math.max(cfg.concurrent_limit || 1, 1), progressPending))
          : 0

      const actionLabel =
        statusCode === 'running'
          ? '暂停'
          : statusCode === 'completed' || (progress.total > 0 && progress.processed + progress.error >= progress.total)
            ? '完结'
            : progress.total > 0 && progress.processed + progress.error < progress.total
              ? '继续'
              : '开始'

      return {
        progress,
        serial: projectPage.value * normalizedProjectPageSize.value + idx + 1,
        name: cfg.name,
        sourceLanguage: resolveLanguageLabel(cfg.source_language),
        targetLanguage: resolveLanguageLabel(cfg.target_language),
        concurrentLimit: cfg.concurrent_limit,
        runStatusLabel:
          statusCode === 'running'
            ? '运行中'
            : statusCode === 'paused'
              ? '已暂停'
              : statusCode === 'completed'
                ? '已完成'
                : '未开始',
        progressProcessed: progress.processed,
        progressError: progress.error,
        progressTotal: progress.total,
        progressPending,
        runningCount,
        exporting: exportingProjectSet.value.has(cfg.name),
        actionLabel,
        selected: selectedProjectSet.value.has(cfg.name),
      }
    }),
  )

  const visiblePages = useVisiblePages(currentPage, totalPages)

  async function bulkDeleteSelected() {
    const names = Array.from(selectedProjectSet.value)
    if (names.length === 0) {
      return
    }
    const ok = window.confirm(`确认删除选中的 ${names.length} 个项目吗？`)
    if (!ok) {
      return
    }
    try {
      const removed = await projectBridge.deleteProjects({ names })
      selectedProjectSet.value = new Set(
        Array.from(selectedProjectSet.value).filter((item) => !names.includes(item)),
      )
      await loadProjects()
      toast.success(`已批量删除 ${removed} 个项目配置`)
    } catch (error) {
      toast.error(`批量删除项目配置失败: ${toErrorMessage(error)}`)
    }
  }

  const detailPageSize = ref(10)
  const detailPage = ref(0)
  const selectedStatuses = ref<string[]>([])
  const availableRunStatuses = ref(['未开始', '运行中', '已暂停', '已完成'])
  const {
    selectedValues: selectedRunStatuses,
    allSelected: allRunStatusesSelected,
    partiallySelected: runStatusesPartiallySelected,
    toggleValue: toggleRunStatus,
    toggleAll: handleToggleAllRunStatuses,
  } = useMultiSelectFilter(availableRunStatuses)

  watch(selectedRunStatuses, () => {
    if (currentPage.value !== 0) {
      updateProjectProfilePage(0)
    }
  })

  watch(
    selectedRunStatuses,
    (value) => {
      projectRunStatusFilters.value = [...value]
    },
    { deep: true },
  )

  watch([totalPages, currentPage], () => {
    const maxPage = Math.max(totalPages.value - 1, 0)
    if (currentPage.value > maxPage) {
      updateProjectProfilePage(maxPage)
    }
  })

  const normalizedDetailItems = computed(() =>
    detailItems.value.map((item) => ({
      ...item,
      normalizedStatus: String(item.status ?? '').trim().toUpperCase() || 'NONE',
    })),
  )

  const availableStatuses = computed(() => {
    const set = new Set<string>()
    for (const item of normalizedDetailItems.value) {
      set.add(item.normalizedStatus)
    }
    return Array.from(set.values())
  })

  watch(
    () => [showDetail.value, detailProjectName.value, detailItems.value.length],
    ([show]) => {
      if (!show) {
        return
      }
      detailPage.value = 0
      detailPageSize.value = 10
      selectedStatuses.value = availableStatuses.value.length > 0 ? [...availableStatuses.value] : []
    },
    { immediate: true },
  )

  const filteredDetailItems = computed(() => {
    const selected = new Set(selectedStatuses.value)
    if (selected.size === 0) {
      return []
    }
    return normalizedDetailItems.value.filter((item) => selected.has(item.normalizedStatus))
  })

  const normalizedDetailPageSize = computed(() => {
    return toPositiveInt(Number(detailPageSize.value), 10)
  })

  const detailTotalPages = computed(() => {
    const total = filteredDetailItems.value.length
    if (total === 0) {
      return 0
    }
    return Math.ceil(total / normalizedDetailPageSize.value)
  })

  const visibleDetailPages = useVisiblePages(detailPage, detailTotalPages)

  const visibleDetailItems = computed(() => {
    const maxPage = Math.max(detailTotalPages.value - 1, 0)
    const page = Math.min(Math.max(detailPage.value, 0), maxPage)
    const start = page * normalizedDetailPageSize.value
    return filteredDetailItems.value.slice(start, start + normalizedDetailPageSize.value)
  })

  const visibleDetailRows = computed(() => {
    const maxPage = Math.max(detailTotalPages.value - 1, 0)
    const page = Math.min(Math.max(detailPage.value, 0), maxPage)
    const start = page * normalizedDetailPageSize.value
    return visibleDetailItems.value.map((item, offset) => ({
      serial: start + offset + 1,
      item,
    }))
  })

  function toggleDetailStatus(status: string) {
    const next = new Set(selectedStatuses.value)
    if (next.has(status)) {
      next.delete(status)
    } else {
      next.add(status)
    }
    selectedStatuses.value = Array.from(next.values())
    detailPage.value = 0
  }

  function goPrevDetailPage() {
    detailPage.value = Math.max(0, detailPage.value - 1)
  }

  function goNextDetailPage() {
    detailPage.value = Math.min(Math.max(detailTotalPages.value - 1, 0), detailPage.value + 1)
  }

  function updateDetailPageSize(value: number) {
    detailPageSize.value = toPositiveInt(value, 10)
    detailPage.value = 0
  }

  function updateDetailPage(value: number) {
    const numeric = Number.isFinite(value) ? Math.floor(value) : 0
    const maxPage = Math.max(detailTotalPages.value - 1, 0)
    detailPage.value = Math.min(Math.max(numeric, 0), maxPage)
  }

  function goToFirstDetailPage() {
    detailPage.value = 0
  }

  function goToLastDetailPage() {
    if (detailTotalPages.value > 0) {
      detailPage.value = detailTotalPages.value - 1
    }
  }

  function goToDetailPage(page: number) {
    const maxPage = Math.max(detailTotalPages.value - 1, 0)
    detailPage.value = Math.min(Math.max(Math.floor(page), 0), maxPage)
  }

  function mapDetailStatusLabel(status: string): string {
    switch (status) {
      case 'PROCESSED':
        return '已完成'
      case 'ERROR':
        return '错误'
      case 'PROCESSING':
        return '处理中'
      case 'EXCLUDED':
        return '已排除'
      default:
        return '未处理'
    }
  }

  function mapDetailActionLabel(status: string): string {
    switch (status) {
      case 'PROCESSED':
        return '重译'
      case 'ERROR':
        return '重试'
      default:
        return '翻译'
    }
  }

  const showTextEditor = ref(false)
  const editingField = ref<'source_text' | 'translated_text'>('source_text')
  const editingItemId = ref('')
  const editingText = ref('')
  const localRunningDetailItemIds = ref<string[]>([])

  function isDetailItemRunning(id: string): boolean {
    return runningDetailItemIds.value.includes(id) || localRunningDetailItemIds.value.includes(id)
  }

  watch(
    () => runningDetailItemIds.value.slice(),
    (runningIds) => {
      localRunningDetailItemIds.value = localRunningDetailItemIds.value.filter((id) =>
        runningIds.includes(id),
      )
    },
  )

  function triggerRunDetailItem(projectName: string, id: string, status: string) {
    if (isDetailItemRunning(id)) {
      return
    }
    localRunningDetailItemIds.value = [...localRunningDetailItemIds.value, id]
    runDetailItemAction({ projectName, id, status })
  }

  function openTextEditor(
    item: { id: string; source_text: string; translated_text: string },
    field: 'source_text' | 'translated_text',
  ) {
    editingItemId.value = item.id
    editingField.value = field
    editingText.value = field === 'source_text' ? item.source_text : item.translated_text
    showTextEditor.value = true
  }

  function closeTextEditor() {
    showTextEditor.value = false
  }

  function saveTextEditor() {
    if (!editingItemId.value) {
      return
    }
    applyDetailItemTextUpdate({
      id: editingItemId.value,
      field: editingField.value,
      value: editingText.value,
    })
    closeTextEditor()
  }

  return {
    languages,
    searchKeyword,
    selectedCount,
    projectConfigs,
    pageSize,
    currentPage,
    totalPages,
    visiblePages,
    showEditor,
    editingProjectName,
    draftProjectName,
    draftInputPath,
    draftOutputPath,
    draftSourceLanguage,
    draftTargetLanguage,
    draftProviderName,
    draftConcurrentLimit,
    draftPrompt,
    draftPromptPresetName,
    promptPresetOptions,
    providerOptions,
    showDetail,
    detailProjectName,
    detailPrompt,
    detailItems,
    runningDetailItemIds,
    availableRunStatuses,
    selectedRunStatuses,
    allRunStatusesSelected,
    runStatusesPartiallySelected,
    handleToggleAllRunStatuses,
    toggleRunStatus,
    openCreateProject,
    selectAllVisibleProjects,
    bulkDeleteSelected,
    updateProjectProfilePageSize,
    updateProjectProfilePage,
    toggleSelectProject,
    startOrPauseProject,
    retryProjectErrors,
    retryProjectAll,
    openProjectDetail,
    exportProject,
    openEditProject,
    deleteProject,
    applyPromptPresetToProject,
    selectInputFolder,
    selectInputDirectory,
    selectOutputFolder,
    saveProjectProfile,
    cancelEditProject,
    closeProjectDetail,
    clearProjectCache,
    availableStatuses,
    selectedStatuses,
    mapDetailStatusLabel,
    normalizedDetailPageSize,
    updateDetailPageSize,
    detailPage,
    updateDetailPage,
    detailTotalPages,
    goToFirstDetailPage,
    goPrevDetailPage,
    visibleDetailPages,
    goToDetailPage,
    goNextDetailPage,
    goToLastDetailPage,
    visibleDetailRows,
    openTextEditor,
    isDetailItemRunning,
    triggerRunDetailItem,
    mapDetailActionLabel,
    showTextEditor,
    editingField,
    editingText,
    closeTextEditor,
    saveTextEditor,
    toggleDetailStatus,
  }
}
