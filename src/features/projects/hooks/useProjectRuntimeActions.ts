import { ref, type Ref } from 'vue'
import { providerBridge } from '../../providers/api/providerBridge'
import { projectBridge } from '../api/projectBridge'
import type {
  Project,
  ProjectStatusCode,
  TranslationItem,
} from '../types'
import { buildFinalProviderPrompt, normalizeProject } from '../utils'
import { toErrorMessage } from '../../../shared/utils/core'

interface ToastLike {
  success: (message: string, options?: { duration?: number }) => void
  error: (message: string) => void
  message: (message: string) => void
}

type ProgressEntry = {
  status: ProjectStatusCode
  total: number
  processed: number
  error: number
  is_running: boolean
}

interface UseProjectRuntimeActionsDeps {
  toast: ToastLike
  loadProjects: () => Promise<void>
  projectLibrary: Ref<Project[]>
  projectProgressMap: Ref<Record<string, ProgressEntry>>
}

type ProjectRunMode = 'start' | 'continue' | 'retry_errors' | 'retry_all'

export function useProjectRuntimeActions(deps: UseProjectRuntimeActionsDeps) {
  const {
    toast,
    loadProjects,
    projectLibrary,
    projectProgressMap,
  } = deps

  const showProjectDetail = ref<boolean>(false)
  const detailProjectName = ref<string>('')
  const detailFinalPrompt = ref<string>('')
  const detailItems = ref<TranslationItem[]>([])
  const runningDetailItemIds = ref<string[]>([])
  const exportingProjectSet = ref<Set<string>>(new Set())

  const activeProjectRunName = ref<string>('')
  const isTranslating = ref<boolean>(false)

  const globalPollTimer = ref<number | null>(null)

  async function refreshAllProjectProgresses() {
    try {
      const snapshots = await projectBridge.getAllProjectRuntimeSnapshots()
      const nextProgressMap: Record<string, ProgressEntry> = {}
      for (const entry of snapshots ?? []) {
        const name = String(entry.name ?? '')
        if (!name) {
          continue
        }
        const status = entry.status ?? 'not_started'
        nextProgressMap[name] = {
          status,
          total: entry.total ?? 0,
          processed: entry.processed ?? 0,
          error: entry.error ?? 0,
          is_running: status === 'running',
        }
      }
      projectProgressMap.value = nextProgressMap
      const runningEntry = Object.entries(nextProgressMap).find(([, snapshot]) => snapshot.is_running)
      if (runningEntry) {
        isTranslating.value = true
        activeProjectRunName.value = runningEntry[0]
        return true
      }
      isTranslating.value = false
      activeProjectRunName.value = ''
      return false
    } catch (error) {
      console.error(`批量同步项目进度失败: ${toErrorMessage(error)}`)
      return false
    }
  }

  function startGlobalProgressPolling() {
    if (globalPollTimer.value !== null) return

    globalPollTimer.value = window.setInterval(async () => {
      if (projectLibrary.value.length === 0) {
        stopGlobalProgressPolling()
        return
      }

      const hasRunningProject = await refreshAllProjectProgresses()
      if (!hasRunningProject) {
        stopGlobalProgressPolling()
      }
    }, 800)
  }

  function stopGlobalProgressPolling() {
    if (globalPollTimer.value !== null) {
      window.clearInterval(globalPollTimer.value)
      globalPollTimer.value = null
    }
  }

  async function ensureProjectBound(
    name: string,
    inputPathValue: string,
    outputPathValue: string,
    sourceLanguageValue: string,
    targetLanguageValue: string,
    providerName: string,
  ) {
    await projectBridge.createProject({
      name,
      inputPath: inputPathValue,
      outputPath: outputPathValue,
      sourceLanguage: sourceLanguageValue,
      targetLanguage: targetLanguageValue,
      providerName,
    })
  }

  async function loadProjectConfig(name: string) {
    const loaded = await projectBridge.getProject({ name })
    return normalizeProject(loaded, name)
  }

  async function loadProjectItems(name: string) {
    return projectBridge.loadProjectItems({ name })
  }

  async function startProjectRun(name: string, mode: ProjectRunMode) {
    const cfg = await loadProjectConfig(name)

    isTranslating.value = true
    activeProjectRunName.value = cfg.name
    startGlobalProgressPolling()

    toast.success(
      mode === 'continue'
        ? '任务继续中...'
        : mode === 'retry_errors'
          ? '错误重试已启动...'
          : mode === 'retry_all'
            ? '全部重试已启动...'
            : '任务已启动...',
      { duration: 1200 },
    )

    await ensureProjectBound(
      cfg.name,
      cfg.input_path,
      cfg.output_path,
      cfg.source_language,
      cfg.target_language,
      cfg.provider_name,
    )

    const provider = await providerBridge.getProvider({ name: cfg.provider_name })

    if (mode === 'start' || mode === 'retry_all') {
      await projectBridge.readInputFiles({ inputPath: cfg.input_path })
    } else {
      const currentItems = await loadProjectItems(cfg.name)
      const filtered =
        mode === 'retry_errors'
          ? currentItems.filter((item) => item.status === 'Error' || item.status === 'ERROR')
          : currentItems.filter((item) => item.status !== 'Processed' && item.status !== 'PROCESSED')
      if (filtered.length === 0) {
        toast.error(mode === 'retry_errors' ? '当前没有错误条目可重试' : '当前没有未完成条目可继续')
        return
      }
      await projectBridge.setItems({ items: filtered })
    }

    try {
      await projectBridge.startTranslation({
        config: provider,
        sourceLanguage: cfg.source_language,
        targetLanguage: cfg.target_language,
      })
      startGlobalProgressPolling()
    } catch (error) {
      isTranslating.value = false
      activeProjectRunName.value = ''
      toast.error(`翻译失败: ${toErrorMessage(error)}`)
    }
  }

  function resetRunState(name: string) {
    stopGlobalProgressPolling()
    isTranslating.value = false
    activeProjectRunName.value = ''
    projectProgressMap.value = {
      ...projectProgressMap.value,
      [name]: {
        ...(projectProgressMap.value[name] ?? {
          status: 'not_started' as ProjectStatusCode,
          total: 0,
          processed: 0,
          error: 0,
          is_running: false,
        }),
      },
    }
  }

  async function startOrPauseProject(name: string) {
    const stats = projectProgressMap.value[name]
    if (activeProjectRunName.value === name && (isTranslating.value || stats?.is_running)) {
      try {
        isTranslating.value = false
        const current = projectProgressMap.value[name]
        projectProgressMap.value = {
          ...projectProgressMap.value,
          [name]: {
            ...current,
            status: (current?.total ?? 0) > 0 && (current?.processed ?? 0) + (current?.error ?? 0) >= (current?.total ?? 0)
              ? 'completed'
              : 'paused',
            is_running: false,
          },
        }
        await projectBridge.stopTranslation()
        toast.success('已暂停', { duration: 1000 })
      } catch (error) {
        toast.error(`暂停失败: ${toErrorMessage(error)}`)
      }
      return
    }

    if (stats && stats.total > 0 && stats.processed + stats.error >= stats.total) {
      toast.success('该任务已完结，可使用"全部重试"重新执行', { duration: 1500 })
      return
    }

    const actionMode =
      stats && stats.total > 0 && stats.processed + stats.error < stats.total ? 'continue' : 'start'
    try {
      await startProjectRun(name, actionMode)
    } catch (error) {
      resetRunState(name)
      toast.error(`启动任务失败: ${toErrorMessage(error)}`)
    }
  }

  async function retryProjectErrors(name: string) {
    try {
      await startProjectRun(name, 'retry_errors')
    } catch (error) {
      resetRunState(name)
      toast.error(`启动错误重试失败: ${toErrorMessage(error)}`)
    }
  }

  async function retryProjectAll(name: string) {
    try {
      await startProjectRun(name, 'retry_all')
    } catch (error) {
      resetRunState(name)
      toast.error(`启动全部重试失败: ${toErrorMessage(error)}`)
    }
  }

  async function openProjectDetail(name: string) {
    try {
      const cfg = await loadProjectConfig(name)
      const loadedItems = await loadProjectItems(cfg.name)
      detailProjectName.value = cfg.name
      detailFinalPrompt.value = buildFinalProviderPrompt(
        cfg.prompt,
        cfg.source_language,
        cfg.target_language,
      )
      detailItems.value = loadedItems
      showProjectDetail.value = true
    } catch (error) {
      toast.error(`加载详情失败: ${toErrorMessage(error)}`)
    }
  }

  function closeProjectDetail() {
    showProjectDetail.value = false
  }

  function updateDetailItemText(payload: {
    id: string
    field: 'source_text' | 'translated_text'
    value: string
  }) {
    detailItems.value = detailItems.value.map((item) => {
      if (item.id !== payload.id) {
        return item
      }
      return {
        ...item,
        [payload.field]: payload.value,
      }
    })
  }

  function sleep(ms: number) {
    return new Promise((resolve) => {
      window.setTimeout(resolve, ms)
    })
  }

  async function runDetailItemCore(currentProjectName: string, id: string) {
    if (isTranslating.value) {
      toast.error('当前有任务运行中，请稍后再试')
      return
    }

    const cfg = await loadProjectConfig(currentProjectName)
    await ensureProjectBound(
      cfg.name,
      cfg.input_path,
      cfg.output_path,
      cfg.source_language,
      cfg.target_language,
      cfg.provider_name,
    )

    const currentItems = await loadProjectItems(cfg.name)
    const found = currentItems.find((item) => item.id === id)
    if (!found) {
      toast.error('未找到对应条目')
      return
    }
    const itemForRun: TranslationItem = {
      ...found,
      error_message: null,
    }

    await projectBridge.setItems({ items: [itemForRun] })
    const provider = await providerBridge.getProvider({ name: cfg.provider_name })
    await projectBridge.startTranslation({
      config: provider,
      sourceLanguage: cfg.source_language,
      targetLanguage: cfg.target_language,
    })

    let finished = false
    for (let i = 0; i < 300; i += 1) {
      const p = await projectBridge.getProgress()
      if (!p.is_running) {
        finished = true
        break
      }
      await sleep(200)
    }
    if (!finished) {
      throw new Error('执行超时，请稍后在详情中刷新查看结果')
    }

    const refreshed = await loadProjectItems(cfg.name)
    detailItems.value = refreshed
    await loadProjects()
  }

  async function runDetailItem(payload: { projectName: string; id: string; status: string }) {
    if (runningDetailItemIds.value.includes(payload.id)) {
      return
    }
    runningDetailItemIds.value = [...runningDetailItemIds.value, payload.id]
    try {
      const normalizedStatus = String(payload.status ?? '').toUpperCase()
      const action =
        normalizedStatus === 'PROCESSED' ? '重译' : normalizedStatus === 'ERROR' ? '重试' : '翻译'
      toast.message(`条目${action}已开始`)
      await runDetailItemCore(payload.projectName, payload.id)
      toast.success(`条目${action}完成`)
    } catch (error) {
      toast.error(`条目执行失败: ${toErrorMessage(error)}`)
    } finally {
      runningDetailItemIds.value = runningDetailItemIds.value.filter((itemId) => itemId !== payload.id)
    }
  }

  async function exportProject(name: string) {
    if (exportingProjectSet.value.has(name)) {
      return
    }
    exportingProjectSet.value = new Set(exportingProjectSet.value).add(name)
    try {
      const cfg = await loadProjectConfig(name)

      await ensureProjectBound(
        cfg.name,
        cfg.input_path,
        cfg.output_path,
        cfg.source_language,
        cfg.target_language,
        cfg.provider_name,
      )
      const loadedItems = await loadProjectItems(cfg.name)
      await projectBridge.setItems({ items: loadedItems })
      const written = await projectBridge.exportFiles({ outputPath: cfg.output_path })
      toast.success(`导出完成，共导出 ${written} 个文件`)
    } catch (error) {
      toast.error(`导出失败: ${toErrorMessage(error)}`)
    } finally {
      const next = new Set(exportingProjectSet.value)
      next.delete(name)
      exportingProjectSet.value = next
    }
  }

  async function clearProjectCache(name: string) {
    const normalizedProjectName = name.trim()
    if (!normalizedProjectName) {
      return
    }
    const stats = projectProgressMap.value[normalizedProjectName]
    const isRunning =
      (activeProjectRunName.value === normalizedProjectName && isTranslating.value) ||
      Boolean(stats?.is_running) ||
      stats?.status === 'running'
    if (isRunning) {
      toast.error('项目运行中，无法清除缓存')
      return
    }
    try {
      const removed = await projectBridge.clearProjectItems({ name: normalizedProjectName })
      if (detailProjectName.value === normalizedProjectName) {
        detailItems.value = []
      }
      await loadProjects()
      toast.success(`缓存已清除，共删除 ${removed} 条`)
    } catch (error) {
      toast.error(`清除缓存失败: ${toErrorMessage(error)}`)
    }
  }

  return {
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
    updateDetailItemText,
    runDetailItem,
    exportProject,
    clearProjectCache,
  }
}
