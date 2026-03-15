import type { Provider } from './types'
import { toNonNegativeInt } from '../../shared/utils/core'
import { DEFAULT_ANTHROPIC_VERSION, DEFAULT_API_MODELS } from '../../shared/constants'
import type { ApiFormatKey } from '../../shared/types/ui'

function normalizeFormat(rawFormat?: Provider['format']): Provider['format'] {
  const fallback: Provider['format'] = { type: 'openai' }
  const formatType = rawFormat?.type ?? fallback.type
  if (formatType === 'anthropic') {
    const version = typeof (rawFormat as { anthropic_version?: string | null })?.anthropic_version === 'string'
      ? (rawFormat as { anthropic_version?: string | null }).anthropic_version?.trim()
      : undefined
    return {
      type: 'anthropic',
      anthropic_version: version && version.length > 0 ? version : DEFAULT_ANTHROPIC_VERSION,
    }
  }
  if (formatType === 'google' || formatType === 'openai') {
    return { type: formatType }
  }
  return fallback
}

export function normalizeBaseUrl(url: string): string {
  return url.trim().replace(/\/+$/, '')
}

export function isOfficialApiBaseUrl(
  url: string,
  officialApiBaseUrl: Record<string, string>,
): boolean {
  const normalized = normalizeBaseUrl(url)
  return Object.values(officialApiBaseUrl).some((item) => normalizeBaseUrl(item) === normalized)
}

export function parseApiKeysFromText(text: string): {
  entries: Array<{ key: string; weight: number }>
  errors: string[]
} {
  const entries: Array<{ key: string; weight: number }> = []
  const errors: string[] = []
  const lines = text.split('\n')
  for (let i = 0; i < lines.length; i += 1) {
    const lineNo = i + 1
    const rawLine = lines[i].trim()
    if (!rawLine) {
      continue
    }
    const normalized = rawLine.replace(/，/g, ',')
    const parts = normalized.split(',').map((v) => v.trim()).filter((v) => v.length > 0)
    if (parts.length === 0) {
      continue
    }
    if (parts.length > 2) {
      errors.push(`第 ${lineNo} 行字段过多，应为 key,weight`)
      continue
    }
    const key = parts[0]
    if (!key) {
      errors.push(`第 ${lineNo} 行 key 为空`)
      continue
    }
    let weight = 1
    if (parts.length === 2) {
      const n = Number(parts[1])
      if (!Number.isFinite(n) || n <= 0) {
        errors.push(`第 ${lineNo} 行 weight 必须是大于 0 的数字`)
        continue
      }
      weight = n
    }
    entries.push({ key, weight })
  }
  return { entries, errors }
}

export function formatApiKeysToText(keys: Array<{ key: string; weight: number }>): string {
  return keys.map((entry) => `${entry.key},${entry.weight}`).join('\n')
}

export function normalizeProvider(config: Partial<Provider>, fallbackName: string): Provider {
  const format = normalizeFormat(config.format)
  const fallbackModel = DEFAULT_API_MODELS[format.type as ApiFormatKey]
  return {
    name: String(config.name ?? fallbackName).trim() || fallbackName,
    format,
    api_url: config.api_url ?? 'https://api.openai.com/v1',
    api_keys: Array.isArray(config.api_keys) ? config.api_keys : [],
    group_strategy: config.group_strategy ?? 'sequential',
    max_retries_per_key: toNonNegativeInt(Number(config.max_retries_per_key ?? 2), 2),
    model: config.model ?? fallbackModel ?? DEFAULT_API_MODELS.openai,
    timeout: toNonNegativeInt(Number(config.timeout ?? 120), 120),
    requests_per_second: toNonNegativeInt(Number(config.requests_per_second ?? 0), 0),
    requests_per_minute: toNonNegativeInt(Number(config.requests_per_minute ?? 60), 60),
    requests_per_hour: toNonNegativeInt(Number(config.requests_per_hour ?? 0), 0),
    requests_per_day: toNonNegativeInt(Number(config.requests_per_day ?? 0), 0),
  }
}
