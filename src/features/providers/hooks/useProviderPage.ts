import { computed, watch } from 'vue'
import { toast } from 'vue-sonner'
import { providerBridge } from '../api/providerBridge'
import { toErrorMessage } from '../../../shared/utils/core'
import type { ApiFormatKey } from '../../../shared/types/ui'
import { useProviders } from './useProviders'
import { usePageDataSync } from '../../../shared/composables/usePageDataSync'
import { useVisiblePages } from '../../../shared/composables/useVisiblePages'

export function useProviderPage() {
  const {
    providerLibrary,
    runningProviderTestNames,
    selectedProviderSet,
    editingProviderName,
    providerSearchKeyword,
    showProviderEditor,
    providerPage,
    providerTotalPages,
    draftProviderName,
    draftApiFormat,
    draftApiUrl,
    draftApiModel,
    draftAnthropicVersion,
    draftApiTimeout,
    draftRequestsPerSecond,
    draftRequestsPerMinute,
    draftRequestsPerHour,
    draftRequestsPerDay,
    draftApiKeysText,
    draftGroupStrategy,
    draftMaxRetriesPerKey,
    normalizedProviderPageSize,
    loadProviders,
    saveProvider,
    cancelEditProvider,
    deleteProvider,
    toggleSelectConfig,
    selectAllVisible,
    testProvider,
    openCreateProvider,
    openEditProvider,
    updateProviderPageSize,
    updateProviderPage,
    syncDraftApiUrlWithFormat,
    syncDraftApiModelWithFormat,
  } = useProviders(toast)

  usePageDataSync({
    toast,
    initialTasks: [{ task: loadProviders, errorPrefix: '加载 Provider 失败' }],
    watchTasks: [
      {
        filterSources: [providerSearchKeyword, normalizedProviderPageSize],
        page: providerPage,
        task: loadProviders,
        errorPrefix: '加载 Provider 失败',
      },
    ],
  })

  watch(draftApiFormat, (_newFormat, oldFormat) => {
    syncDraftApiUrlWithFormat()
    syncDraftApiModelWithFormat(oldFormat as ApiFormatKey | undefined)
  })

  const selectedCount = computed(() => selectedProviderSet.value.size)
  const pageSize = normalizedProviderPageSize
  const currentPage = providerPage
  const totalPages = computed(() => providerTotalPages.value)
  const showEditor = showProviderEditor

  const visiblePages = useVisiblePages(currentPage, totalPages)

  const providers = computed(() =>
    providerLibrary.value.map((cfg, idx) => ({
      serial: providerPage.value * normalizedProviderPageSize.value + idx + 1,
      name: cfg.name,
      formatType: cfg.format.type,
      keyCount: cfg.api_keys.length,
      selected: selectedProviderSet.value.has(cfg.name),
    })),
  )

  watch([totalPages, currentPage], () => {
    const maxPage = Math.max(totalPages.value - 1, 0)
    if (currentPage.value > maxPage) {
      updateProviderPage(maxPage)
    }
  })

  async function bulkDeleteSelected() {
    const names = Array.from(selectedProviderSet.value)
    if (names.length === 0) {
      return
    }
    const ok = window.confirm(`确认删除选中的 ${names.length} 个 Provider 吗？`)
    if (!ok) {
      return
    }
    try {
      const removed = await providerBridge.deleteProviders({ names })
      selectedProviderSet.value = new Set(
        Array.from(selectedProviderSet.value).filter((item) => !names.includes(item)),
      )
      await loadProviders()
      toast.success(`已批量删除 ${removed} 个 Provider`)
    } catch (error) {
      toast.error(`批量删除 Provider 失败: ${toErrorMessage(error)}`)
    }
  }

  return {
    providerSearchKeyword,
    openCreateProvider,
    selectAllVisible,
    selectedCount,
    bulkDeleteSelected,
    showEditor,
    pageSize,
    updateProviderPageSize,
    currentPage,
    updateProviderPage,
    totalPages,
    visiblePages,
    providers,
    toggleSelectConfig,
    openEditProvider,
    deleteProvider,
    runningProviderTestNames,
    testProvider,
    editingProviderName,
    draftProviderName,
    draftApiFormat,
    draftApiUrl,
    draftApiModel,
    draftAnthropicVersion,
    draftApiTimeout,
    draftRequestsPerSecond,
    draftRequestsPerMinute,
    draftRequestsPerHour,
    draftRequestsPerDay,
    draftApiKeysText,
    draftGroupStrategy,
    draftMaxRetriesPerKey,
    saveProvider,
    cancelEditProvider,
  }
}
