import { invoke } from '@tauri-apps/api/core'
import type {
  Project,
  ProjectRuntimeSnapshot,
  TranslationItem,
  TranslationProgress,
} from '../../features/projects/types'
import type { PromptPreset } from '../../features/presets/types'
import type { Provider } from '../../features/providers/types'
import type { ThemeInput, ThemePreferences, ThemeState } from '../types/theme'

export async function tauriInvoke<T>(command: string, payload?: Record<string, unknown>): Promise<T> {
  return invoke<T>(command, payload)
}

type PagedResult<T> = {
  items: T[]
  total_pages?: number
  page?: number
}

type ProviderLikeConfig = Provider & {
  temperature?: number
}

type TauriCommandMap = {
  query_providers: {
    payload: { keyword: string | null; page: number; pageSize: number }
    result: PagedResult<Provider>
  }
  list_prompt_presets: { payload: undefined; result: PromptPreset[] }
  delete_projects: { payload: { names: string[] }; result: number }
  query_projects: {
    payload: { keyword: string | null; page: number; pageSize: number }
    result: PagedResult<Project>
  }
  get_all_project_runtime_snapshots: { payload: undefined; result: ProjectRuntimeSnapshot[] }
  create_project_profile: { payload: { config: Project }; result: void }
  update_project_profile: { payload: { originalName: string; config: Project }; result: void }
  get_project: { payload: { name: string }; result: Project }
  delete_project: { payload: { name: string }; result: void }
  query_prompt_presets: {
    payload: { keyword: string | null; page: number; pageSize: number }
    result: PagedResult<PromptPreset>
  }
  get_prompt_preset: { payload: { name: string }; result: PromptPreset }
  update_prompt_preset: { payload: { originalName: string; preset: PromptPreset }; result: void }
  create_prompt_preset: { payload: { preset: PromptPreset }; result: void }
  delete_prompt_preset: { payload: { name: string }; result: void }
  delete_providers: { payload: { names: string[] }; result: number }
  get_provider: { payload: { name: string }; result: Provider }
  create_provider: { payload: { config: Provider }; result: void }
  update_provider: { payload: { originalName: string; config: Provider }; result: void }
  delete_provider: { payload: { name: string }; result: void }
  test_provider: { payload: { name: string }; result: string }
  delete_prompt_presets: { payload: { names: string[] }; result: number }
  create_project: {
    payload: {
      name: string
      inputPath: string
      outputPath: string
      sourceLanguage: string
      targetLanguage: string
      providerName: string
    }
    result: Project
  }
  read_input_files: { payload: { inputPath: string }; result: TranslationItem[] }
  load_project_items: { payload: { name: string }; result: TranslationItem[] }
  set_items: { payload: { items: TranslationItem[] }; result: void }
  start_translation: {
    payload: { config: Provider; sourceLanguage: string; targetLanguage: string }
    result: TranslationProgress
  }
  stop_translation: { payload: undefined; result: void }
  get_progress: { payload: undefined; result: TranslationProgress }
  export_files: { payload: { outputPath: string }; result: number }
  clear_project_items: { payload: { name: string }; result: number }
  fetch_models: { payload: { config: ProviderLikeConfig }; result: string[] }
  get_theme_state: { payload: undefined; result: ThemeState }
  create_theme: { payload: { config: ThemeInput }; result: ThemeState }
  update_theme: { payload: { id: number; config: ThemeInput }; result: ThemeState }
  delete_theme: { payload: { id: number }; result: ThemeState }
  restore_default_themes: { payload: undefined; result: ThemeState }
  save_theme_preferences: { payload: { prefs: ThemePreferences }; result: ThemePreferences }
}

type CommandName = keyof TauriCommandMap
type CommandPayload<T extends CommandName> = TauriCommandMap[T]['payload']
type CommandResult<T extends CommandName> = TauriCommandMap[T]['result']

async function invokeCommand<T extends CommandName>(
  command: T,
  payload?: CommandPayload<T>,
): Promise<CommandResult<T>> {
  return invoke<CommandResult<T>>(command, payload as Record<string, unknown> | undefined)
}

export const api = {
  queryProviders: (payload: CommandPayload<'query_providers'>) => invokeCommand('query_providers', payload),
  listPromptPresets: () => invokeCommand('list_prompt_presets'),
  deleteProjects: (payload: CommandPayload<'delete_projects'>) =>
    invokeCommand('delete_projects', payload),
  queryProjects: (payload: CommandPayload<'query_projects'>) =>
    invokeCommand('query_projects', payload),
  getAllProjectRuntimeSnapshots: () => invokeCommand('get_all_project_runtime_snapshots'),
  createProjectProfile: (payload: CommandPayload<'create_project_profile'>) =>
    invokeCommand('create_project_profile', payload),
  updateProjectProfile: (payload: CommandPayload<'update_project_profile'>) =>
    invokeCommand('update_project_profile', payload),
  getProject: (payload: CommandPayload<'get_project'>) => invokeCommand('get_project', payload),
  deleteProject: (payload: CommandPayload<'delete_project'>) =>
    invokeCommand('delete_project', payload),
  queryPromptPresets: (payload: CommandPayload<'query_prompt_presets'>) =>
    invokeCommand('query_prompt_presets', payload),
  getPromptPreset: (payload: CommandPayload<'get_prompt_preset'>) => invokeCommand('get_prompt_preset', payload),
  updatePromptPreset: (payload: CommandPayload<'update_prompt_preset'>) =>
    invokeCommand('update_prompt_preset', payload),
  createPromptPreset: (payload: CommandPayload<'create_prompt_preset'>) =>
    invokeCommand('create_prompt_preset', payload),
  deletePromptPreset: (payload: CommandPayload<'delete_prompt_preset'>) =>
    invokeCommand('delete_prompt_preset', payload),
  deleteProviders: (payload: CommandPayload<'delete_providers'>) => invokeCommand('delete_providers', payload),
  getProvider: (payload: CommandPayload<'get_provider'>) => invokeCommand('get_provider', payload),
  createProvider: (payload: CommandPayload<'create_provider'>) => invokeCommand('create_provider', payload),
  updateProvider: (payload: CommandPayload<'update_provider'>) => invokeCommand('update_provider', payload),
  deleteProvider: (payload: CommandPayload<'delete_provider'>) => invokeCommand('delete_provider', payload),
  testProvider: (payload: CommandPayload<'test_provider'>) => invokeCommand('test_provider', payload),
  deletePromptPresets: (payload: CommandPayload<'delete_prompt_presets'>) =>
    invokeCommand('delete_prompt_presets', payload),
  createProject: (payload: CommandPayload<'create_project'>) => invokeCommand('create_project', payload),
  readInputFiles: (payload: CommandPayload<'read_input_files'>) => invokeCommand('read_input_files', payload),
  loadProjectItems: (payload: CommandPayload<'load_project_items'>) => invokeCommand('load_project_items', payload),
  setItems: (payload: CommandPayload<'set_items'>) => invokeCommand('set_items', payload),
  startTranslation: (payload: CommandPayload<'start_translation'>) => invokeCommand('start_translation', payload),
  stopTranslation: () => invokeCommand('stop_translation'),
  getProgress: () => invokeCommand('get_progress'),
  exportFiles: (payload: CommandPayload<'export_files'>) => invokeCommand('export_files', payload),
  clearProjectItems: (payload: CommandPayload<'clear_project_items'>) => invokeCommand('clear_project_items', payload),
  fetchModels: (payload: CommandPayload<'fetch_models'>) => invokeCommand('fetch_models', payload),
  getThemeState: () => invokeCommand('get_theme_state'),
  createTheme: (payload: CommandPayload<'create_theme'>) => invokeCommand('create_theme', payload),
  updateTheme: (payload: CommandPayload<'update_theme'>) => invokeCommand('update_theme', payload),
  deleteTheme: (payload: CommandPayload<'delete_theme'>) => invokeCommand('delete_theme', payload),
  restoreDefaultThemes: () => invokeCommand('restore_default_themes'),
  saveThemePreferences: (payload: CommandPayload<'save_theme_preferences'>) =>
    invokeCommand('save_theme_preferences', payload),
}
