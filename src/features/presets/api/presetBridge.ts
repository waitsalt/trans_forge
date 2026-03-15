import { api } from '../../../shared/tauri/api'

export const presetBridge = {
  listPromptPresets: api.listPromptPresets,
  queryPromptPresets: api.queryPromptPresets,
  getPromptPreset: api.getPromptPreset,
  createPromptPreset: api.createPromptPreset,
  updatePromptPreset: api.updatePromptPreset,
  deletePromptPreset: api.deletePromptPreset,
  deletePromptPresets: api.deletePromptPresets,
}
