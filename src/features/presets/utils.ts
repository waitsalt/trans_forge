import type { PromptPreset } from './types'

export function normalizePromptPreset(
  preset: Partial<PromptPreset>,
  fallbackName: string,
): PromptPreset {
  return {
    name: String(preset.name ?? fallbackName).trim() || fallbackName,
    language: String(preset.language ?? 'ZH').trim().toUpperCase() || 'ZH',
    prompt: String(preset.prompt ?? ''),
  }
}
