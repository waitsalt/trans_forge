import type { Project, ProjectStatusCode } from './types'
import { toNonNegativeInt } from '../../shared/utils/core'

export function normalizeProject(
  config: Partial<Project>,
  fallbackName: string,
): Project {
  const runStatusRaw = String(config.run_status ?? 'not_started').trim().toLowerCase()
  const runStatus: ProjectStatusCode =
    runStatusRaw === 'running'
      ? 'running'
      : runStatusRaw === 'paused'
        ? 'paused'
        : runStatusRaw === 'completed'
          ? 'completed'
          : 'not_started'
  return {
    name: String(config.name ?? fallbackName).trim() || fallbackName,
    source_language: String(config.source_language ?? 'JA'),
    target_language: String(config.target_language ?? 'ZH'),
    provider_name: String(config.provider_name ?? ''),
    concurrent_limit: toNonNegativeInt(Number(config.concurrent_limit ?? 1), 1) || 1,
    prompt: typeof config.prompt === 'string' ? config.prompt : null,
    run_status: runStatus,
    input_path: String(config.input_path ?? ''),
    output_path: String(config.output_path ?? ''),
  }
}

export function mapRunStatusLabelToCode(label: string): ProjectStatusCode | null {
  if (label === '运行中') {
    return 'running'
  }
  if (label === '已完成') {
    return 'completed'
  }
  if (label === '已暂停') {
    return 'paused'
  }
  if (label === '未开始') {
    return 'not_started'
  }
  return null
}

export function buildFinalProviderPrompt(
  projectPrompt: string | null | undefined,
  sourceLang: string,
  targetLang: string,
): string {
  const normalizedPrompt = typeof projectPrompt === 'string' ? projectPrompt.trim() : ''
  if (normalizedPrompt.length > 0) {
    return normalizedPrompt.replace(/\{source\}/g, sourceLang).replace(/\{target\}/g, targetLang)
  }
  return `You are a professional translator. Translate the following text from ${sourceLang} to ${targetLang}. Only output the translated text, no explanations.`
}
