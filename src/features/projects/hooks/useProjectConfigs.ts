import { computed, ref, type Ref } from 'vue'
import { projectBridge } from '../api/projectBridge'
import type { PromptPreset } from '../../presets/types'
import type { Project } from '../types'
import { normalizeProject } from '../utils'
import { toErrorMessage, toNonNegativeInt, toPositiveInt } from '../../../shared/utils/core'

interface ToastLike {
  success: (message: string) => void
  error: (message: string) => void
}

type ProgressEntry = {
  status: string
  total: number
  processed: number
  error: number
  is_running: boolean
}

export function useProjectConfigs(
  toast: ToastLike,
  projectProgressMap: Ref<Record<string, ProgressEntry>>,
  promptPresetAll: Ref<PromptPreset[]>,
) {
  const projectLibrary = ref<Project[]>([])
  const projectSearchKeyword = ref<string>('')
  const selectedProjectSet = ref<Set<string>>(new Set())
  const showProjectEditor = ref<boolean>(false)
  const editingProjectName = ref<string | null>(null)
  const projectPageSize = ref<number>(10)
  const projectPage = ref<number>(0)
  const projectTotalPages = ref<number>(0)
  const draftProjectName = ref<string>('')
  const draftProjectSourceLanguage = ref<string>('JA')
  const draftProjectTargetLanguage = ref<string>('ZH')
  const draftProjectProviderName = ref<string>('')
  const draftProjectConcurrentLimit = ref<number>(1)
  const draftProjectPrompt = ref<string | null>(null)
  const draftProjectPromptPresetName = ref<string>('')
  const draftProjectInputPath = ref<string>('')
  const draftProjectOutputPath = ref<string>('')

  const normalizedProjectPageSize = computed(() => {
    return toPositiveInt(Number(projectPageSize.value), 10)
  })

  async function loadProjects() {
    const keyword = projectSearchKeyword.value.trim()
    const response = await projectBridge.queryProjects({
      keyword: keyword.length > 0 ? keyword : null,
      page: projectPage.value,
      pageSize: normalizedProjectPageSize.value,
    })
    const normalized = (response.items ?? []).map((item, idx) =>
      normalizeProject(item, item.name?.trim() || `project-${idx + 1}`),
    )
    projectLibrary.value = normalized

    const snapshotEntries = await projectBridge.getAllProjectRuntimeSnapshots()
    const nextProgressMap = { ...projectProgressMap.value }
    for (const entry of snapshotEntries ?? []) {
      const name = String(entry.name ?? '')
      if (!name) {
        continue
      }
      nextProgressMap[name] = {
        status: entry.status ?? 'not_started',
        total: entry.total ?? 0,
        processed: entry.processed ?? 0,
        error: entry.error ?? 0,
        is_running: entry.status === 'running',
      }
    }
    projectProgressMap.value = nextProgressMap
    projectTotalPages.value = response.total_pages ?? 0
    projectPage.value = response.page ?? 0
  }

  function fillDraftFromProject(config: Project) {
    draftProjectName.value = config.name
    draftProjectSourceLanguage.value = config.source_language
    draftProjectTargetLanguage.value = config.target_language
    draftProjectProviderName.value = config.provider_name
    draftProjectConcurrentLimit.value = toNonNegativeInt(Number(config.concurrent_limit ?? 1), 1) || 1
    draftProjectPrompt.value = typeof config.prompt === 'string' ? config.prompt : null
    const matchedPreset = promptPresetAll.value.find(
      (preset) => preset.prompt.trim() === (config.prompt ?? '').trim(),
    )
    draftProjectPromptPresetName.value = matchedPreset?.name ?? ''
    draftProjectInputPath.value = config.input_path
    draftProjectOutputPath.value = config.output_path
  }

  function applyPromptPresetToProject(name: string) {
    draftProjectPromptPresetName.value = name
    const preset = promptPresetAll.value.find((item) => item.name === name)
    if (!preset) {
      return
    }
    draftProjectPrompt.value = preset.prompt
  }

  function buildDraftProject(): Project | null {
    const name = draftProjectName.value.trim()
    if (!name) {
      toast.error('项目配置名称不能为空')
      return null
    }
    if (!draftProjectInputPath.value.trim() || !draftProjectOutputPath.value.trim()) {
      toast.error('请先填写输入路径和输出路径')
      return null
    }
    if (!draftProjectProviderName.value.trim()) {
      toast.error('请选择 Provider')
      return null
    }
    return {
      name,
      source_language: draftProjectSourceLanguage.value,
      target_language: draftProjectTargetLanguage.value,
      provider_name: draftProjectProviderName.value.trim(),
      concurrent_limit: Math.max(1, toNonNegativeInt(Number(draftProjectConcurrentLimit.value), 1)),
      prompt: draftProjectPrompt.value,
      run_status: 'not_started',
      input_path: draftProjectInputPath.value.trim(),
      output_path: draftProjectOutputPath.value.trim(),
    }
  }

  async function createProjectProfile() {
    const next = buildDraftProject()
    if (!next) {
      return
    }
    try {
      await projectBridge.createProjectProfile({ config: next })
      await loadProjects()
      toast.success(`已新增项目配置: ${next.name}`)
      showProjectEditor.value = false
    } catch (error) {
      toast.error(`新增项目配置失败: ${toErrorMessage(error)}`)
    }
  }

  async function updateProjectProfile() {
    const next = buildDraftProject()
    if (!next) {
      return
    }
    const currentName = editingProjectName.value
    if (!currentName) {
      toast.error('未找到要编辑的项目配置')
      return
    }
    try {
      await projectBridge.updateProjectProfile({ originalName: currentName, config: next })
      if (selectedProjectSet.value.delete(currentName)) {
        selectedProjectSet.value.add(next.name)
      }
      await loadProjects()
      toast.success(`已更新项目配置: ${next.name}`)
      showProjectEditor.value = false
    } catch (error) {
      toast.error(`更新项目配置失败: ${toErrorMessage(error)}`)
    }
  }

  async function saveProjectProfile() {
    if (editingProjectName.value) {
      await updateProjectProfile()
    } else {
      await createProjectProfile()
    }
  }

  function openCreateProject(defaultProviderName: string) {
    showProjectEditor.value = true
    editingProjectName.value = null
    draftProjectName.value = ''
    draftProjectSourceLanguage.value = 'JA'
    draftProjectTargetLanguage.value = 'ZH'
    draftProjectProviderName.value = defaultProviderName
    draftProjectConcurrentLimit.value = 1
    draftProjectPrompt.value = null
    draftProjectPromptPresetName.value = ''
    draftProjectInputPath.value = ''
    draftProjectOutputPath.value = ''
  }

  async function openEditProject(name: string) {
    showProjectEditor.value = true
    editingProjectName.value = name
    try {
      const loaded = await projectBridge.getProject({ name })
      fillDraftFromProject(normalizeProject(loaded, name))
    } catch (error) {
      toast.error(`加载项目配置失败: ${toErrorMessage(error)}`)
    }
  }

  function cancelEditProject() {
    showProjectEditor.value = false
    editingProjectName.value = null
  }

  function toggleSelectProject(name: string) {
    const next = new Set(selectedProjectSet.value)
    if (next.has(name)) {
      next.delete(name)
    } else {
      next.add(name)
    }
    selectedProjectSet.value = next
  }

  function selectAllVisibleProjects() {
    const next = new Set(selectedProjectSet.value)
    for (const cfg of projectLibrary.value) {
      next.add(cfg.name)
    }
    selectedProjectSet.value = next
  }

  async function deleteProject(name: string) {
    if (!name) {
      return
    }
    try {
      await projectBridge.deleteProject({ name })
      selectedProjectSet.value.delete(name)
      await loadProjects()
      toast.success(`已删除项目配置: ${name}`)
    } catch (error) {
      toast.error(`删除项目配置失败: ${toErrorMessage(error)}`)
    }
  }

  function updateProjectProfilePageSize(value: number) {
    projectPageSize.value = toPositiveInt(value, 10)
  }

  function updateProjectProfilePage(value: number) {
    const numeric = Number.isFinite(value) ? Math.floor(value) : 0
    const maxPage = Math.max(projectTotalPages.value - 1, 0)
    projectPage.value = Math.min(Math.max(numeric, 0), maxPage)
  }

  function goToFirstProjectPage() {
    projectPage.value = 0
  }

  function goToPrevProjectPage() {
    projectPage.value = Math.max(projectPage.value - 1, 0)
  }

  function goToLastProjectPage() {
    if (projectTotalPages.value > 0) {
      projectPage.value = projectTotalPages.value - 1
    }
  }

  function goToNextProjectPage() {
    const maxPage = Math.max(projectTotalPages.value - 1, 0)
    projectPage.value = Math.min(projectPage.value + 1, maxPage)
  }

  function goToProjectPage(page: number) {
    const maxPage = Math.max(projectTotalPages.value - 1, 0)
    projectPage.value = Math.min(Math.max(Math.floor(page), 0), maxPage)
  }

  return {
    projectLibrary,
    projectSearchKeyword,
    selectedProjectSet,
    showProjectEditor,
    editingProjectName,
    projectPageSize,
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
    openCreateProject,
    openEditProject,
    cancelEditProject,
    toggleSelectProject,
    selectAllVisibleProjects,
    deleteProject,
    updateProjectProfilePageSize,
    updateProjectProfilePage,
    goToFirstProjectPage,
    goToPrevProjectPage,
    goToLastProjectPage,
    goToNextProjectPage,
    goToProjectPage,
  }
}
