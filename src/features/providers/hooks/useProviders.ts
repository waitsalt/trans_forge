import { computed, ref, type Ref } from 'vue'
import { DEFAULT_ANTHROPIC_VERSION, DEFAULT_API_MODELS, OFFICIAL_API_BASE_URL } from '../../../shared/constants'
import type { ApiFormatKey } from '../../../shared/types/ui'
import { toErrorMessage, toNonNegativeInt, toPositiveInt } from '../../../shared/utils/core'
import { providerBridge } from '../api/providerBridge'
import type { Provider } from '../types'
import {
  formatApiKeysToText,
  isOfficialApiBaseUrl,
  normalizeProvider,
  parseApiKeysFromText,
} from '../utils'

interface ToastLike {
  success: (message: string) => void
  error: (message: string) => void
  message: (message: string) => void
}

export function useProviders(toast: ToastLike) {
  const providerLibrary = ref<Provider[]>([])
  const runningProviderTestNames = ref<string[]>([])
  const selectedProviderSet = ref<Set<string>>(new Set())
  const selectedProviderName = ref<string>('')
  const editingProviderName = ref<string | null>(null)
  const providerSearchKeyword = ref<string>('')
  const showProviderEditor = ref<boolean>(false)
  const providerPageSize = ref<number>(10)
  const providerPage = ref<number>(0)
  const providerTotalPages = ref<number>(0)
  const draftProviderName = ref<string>('default')
  const draftApiFormat = ref<ApiFormatKey>('openai')
  const draftApiUrl = ref<string>('https://api.openai.com/v1')
  const draftApiModel = ref<string>(DEFAULT_API_MODELS.openai)
  const draftAnthropicVersion = ref<string>(DEFAULT_ANTHROPIC_VERSION)
  const draftApiTimeout = ref<number>(120)
  const draftRequestsPerSecond = ref<number>(0)
  const draftRequestsPerMinute = ref<number>(60)
  const draftRequestsPerHour = ref<number>(0)
  const draftRequestsPerDay = ref<number>(0)
  const draftApiKeysText = ref<string>('')
  const draftGroupStrategy = ref<string>('sequential')
  const draftMaxRetriesPerKey = ref<number>(2)

  const normalizedProviderPageSize = computed(() => {
    return toPositiveInt(Number(providerPageSize.value), 10)
  })

  function fillDraftFromProvider(config: Provider) {
    draftProviderName.value = config.name
    draftApiFormat.value = config.format.type
    draftApiUrl.value = config.api_url
    draftApiKeysText.value = formatApiKeysToText(config.api_keys)
    draftGroupStrategy.value = config.group_strategy
    draftMaxRetriesPerKey.value = config.max_retries_per_key
    draftApiModel.value = config.model
    draftAnthropicVersion.value =
      config.format.type === 'anthropic'
        ? (config.format.anthropic_version?.trim() || DEFAULT_ANTHROPIC_VERSION)
        : DEFAULT_ANTHROPIC_VERSION
    draftApiTimeout.value = config.timeout
    draftRequestsPerSecond.value = config.requests_per_second
    draftRequestsPerMinute.value = config.requests_per_minute
    draftRequestsPerHour.value = config.requests_per_hour
    draftRequestsPerDay.value = config.requests_per_day
  }

  function buildDraftProvider(): Provider | null {
    const name = draftProviderName.value.trim()
    if (!name) {
      toast.error('Provider 名称不能为空')
      return null
    }
    const parsed = parseApiKeysFromText(draftApiKeysText.value)
    if (parsed.errors.length > 0) {
      toast.error(`API Key 格式错误: ${parsed.errors[0]}`)
      return null
    }
    const parsedKeys = parsed.entries
    if (parsedKeys.length === 0) {
      toast.error('请至少填写一个 API Key（每行：key,weight）')
      return null
    }
    return {
      name,
      format: buildFormatConfig(),
      api_url: draftApiUrl.value.trim(),
      api_keys: parsedKeys,
      group_strategy: draftGroupStrategy.value,
      max_retries_per_key: toNonNegativeInt(draftMaxRetriesPerKey.value, 2),
      model: draftApiModel.value.trim(),
      timeout: toNonNegativeInt(draftApiTimeout.value, 120),
      requests_per_second: toNonNegativeInt(draftRequestsPerSecond.value, 0),
      requests_per_minute: toNonNegativeInt(draftRequestsPerMinute.value, 60),
      requests_per_hour: toNonNegativeInt(draftRequestsPerHour.value, 0),
      requests_per_day: toNonNegativeInt(draftRequestsPerDay.value, 0),
    }
  }

  async function loadProviders() {
    const keyword = providerSearchKeyword.value.trim()
    const response = await providerBridge.queryProviders({
      keyword: keyword.length > 0 ? keyword : null,
      page: providerPage.value,
      pageSize: normalizedProviderPageSize.value,
    })
    const normalized = (response.items ?? []).map((item, idx) =>
      normalizeProvider(item, item.name?.trim() || `config-${idx + 1}`),
    )
    providerLibrary.value = normalized
    providerTotalPages.value = response.total_pages ?? 0
    providerPage.value = response.page ?? 0
  }

  async function selectProvider(name: string) {
    const loaded = await providerBridge.getProvider({ name })
    const found = normalizeProvider(loaded, name)
    selectedProviderName.value = found.name
    editingProviderName.value = found.name
    fillDraftFromProvider(found)
  }

  async function createProvider() {
    const next = buildDraftProvider()
    if (!next) {
      return
    }
    try {
      await providerBridge.createProvider({ config: next })
      selectedProviderName.value = next.name
      editingProviderName.value = next.name
      await loadProviders()
      toast.success(`已新增 Provider: ${next.name}`)
      showProviderEditor.value = false
    } catch (error) {
      toast.error(`新增配置失败: ${toErrorMessage(error)}`)
    }
  }

  async function updateProvider() {
    const next = buildDraftProvider()
    if (!next) {
      return
    }
    const currentName = editingProviderName.value ?? selectedProviderName.value
    if (!currentName) {
      toast.error('未找到要编辑的 Provider')
      return
    }
    try {
      await providerBridge.updateProvider({ originalName: currentName, config: next })
      if (selectedProviderSet.value.delete(currentName)) {
        selectedProviderSet.value.add(next.name)
      }
      selectedProviderName.value = next.name
      editingProviderName.value = next.name
      await loadProviders()
      toast.success(`已更新 Provider: ${next.name}`)
      showProviderEditor.value = false
    } catch (error) {
      toast.error(`更新配置失败: ${toErrorMessage(error)}`)
    }
  }

  async function saveProvider() {
    if (editingProviderName.value) {
      await updateProvider()
    } else {
      await createProvider()
    }
  }

  function cancelEditProvider() {
    showProviderEditor.value = false
    editingProviderName.value = null
  }

  async function deleteProvider(name: string) {
    if (!name) {
      return
    }
    try {
      await providerBridge.deleteProvider({ name })
      selectedProviderSet.value.delete(name)
      if (selectedProviderName.value === name) {
        selectedProviderName.value = ''
      }
      if (editingProviderName.value === name) {
        editingProviderName.value = null
      }
      await loadProviders()
      toast.success(`已删除 Provider: ${name}`)
    } catch (error) {
      toast.error(`删除配置失败: ${toErrorMessage(error)}`)
    }
  }

  function toggleSelectConfig(name: string) {
    const next = new Set(selectedProviderSet.value)
    if (next.has(name)) {
      next.delete(name)
    } else {
      next.add(name)
    }
    selectedProviderSet.value = next
  }

  function selectAllVisible() {
    const next = new Set(selectedProviderSet.value)
    for (const cfg of providerLibrary.value) {
      next.add(cfg.name)
    }
    selectedProviderSet.value = next
  }

  async function testProvider(name: string) {
    if (!name.trim()) {
      return
    }
    if (runningProviderTestNames.value.includes(name)) {
      return
    }
    runningProviderTestNames.value = [...runningProviderTestNames.value, name]
    try {
      toast.message(`开始测试: ${name}`)
      const output = await providerBridge.testProvider({ name })
      toast.success(`测试成功: ${output}`)
    } catch (error) {
      toast.error(`测试失败: ${toErrorMessage(error)}`)
    } finally {
      runningProviderTestNames.value = runningProviderTestNames.value.filter((item) => item !== name)
    }
  }

  function openCreateProvider() {
    showProviderEditor.value = true
    editingProviderName.value = null
    draftProviderName.value = ''
    draftApiFormat.value = 'openai'
    draftApiUrl.value = 'https://api.openai.com/v1'
    draftApiModel.value = DEFAULT_API_MODELS.openai
    draftAnthropicVersion.value = DEFAULT_ANTHROPIC_VERSION
    draftApiTimeout.value = 120
    draftRequestsPerSecond.value = 0
    draftRequestsPerMinute.value = 60
    draftRequestsPerHour.value = 0
    draftRequestsPerDay.value = 0
    draftApiKeysText.value = ''
    draftGroupStrategy.value = 'sequential'
    draftMaxRetriesPerKey.value = 2
  }

  function openEditProvider(name: string) {
    showProviderEditor.value = true
    selectProvider(name).catch((error) => {
      toast.error(`加载配置失败: ${toErrorMessage(error)}`)
    })
  }

  function updateProviderPageSize(value: number) {
    providerPageSize.value = toPositiveInt(value, 10)
  }

  function updateProviderPage(value: number) {
    const numeric = Number.isFinite(value) ? Math.floor(value) : 0
    const maxPage = Math.max(providerTotalPages.value - 1, 0)
    providerPage.value = Math.min(Math.max(numeric, 0), maxPage)
  }

  function goToFirstProviderPage() {
    providerPage.value = 0
  }

  function goToPrevProviderPage() {
    providerPage.value = Math.max(providerPage.value - 1, 0)
  }

  function goToLastProviderPage() {
    if (providerTotalPages.value > 0) {
      providerPage.value = providerTotalPages.value - 1
    }
  }

  function goToNextProviderPage() {
    const maxPage = Math.max(providerTotalPages.value - 1, 0)
    providerPage.value = Math.min(providerPage.value + 1, maxPage)
  }

  function goToProviderPage(page: number) {
    const maxPage = Math.max(providerTotalPages.value - 1, 0)
    providerPage.value = Math.min(Math.max(Math.floor(page), 0), maxPage)
  }

  function syncDraftApiUrlWithFormat() {
    const current = draftApiUrl.value
    if (!current.trim() || isOfficialApiBaseUrl(current, OFFICIAL_API_BASE_URL)) {
      const next = OFFICIAL_API_BASE_URL[draftApiFormat.value as ApiFormatKey]
      if (next) {
        draftApiUrl.value = next
      }
    }
  }

  function syncDraftApiModelWithFormat(previousFormat?: ApiFormatKey) {
    const nextKey = (draftApiFormat.value as ApiFormatKey) || 'openai'
    const nextModel = DEFAULT_API_MODELS[nextKey]
    if (!nextModel) {
      return
    }
    const trimmed = draftApiModel.value.trim()
    const prevDefault = previousFormat ? DEFAULT_API_MODELS[previousFormat] : undefined
    if (!trimmed || (prevDefault && trimmed === prevDefault)) {
      draftApiModel.value = nextModel
    }
  }

  function buildFormatConfig(): Provider['format'] {
    const formatKey = (draftApiFormat.value as ApiFormatKey) || 'openai'
    if (formatKey === 'anthropic') {
      const version = draftAnthropicVersion.value.trim()
      return {
        type: 'anthropic',
        anthropic_version: version || DEFAULT_ANTHROPIC_VERSION,
      }
    }
    return { type: formatKey }
  }

  return {
    providerLibrary,
    runningProviderTestNames,
    selectedProviderSet,
    selectedProviderName,
    editingProviderName,
    providerSearchKeyword,
    showProviderEditor,
    providerPageSize,
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
    goToFirstProviderPage,
    goToPrevProviderPage,
    goToLastProviderPage,
    goToNextProviderPage,
    goToProviderPage,
    syncDraftApiUrlWithFormat,
    syncDraftApiModelWithFormat,
  }
}

export type UseProvidersReturn = ReturnType<typeof useProviders>

export function clearProviderSelections(selectedProviderSet: Ref<Set<string>>) {
  selectedProviderSet.value = new Set()
}
