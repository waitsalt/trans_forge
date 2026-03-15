import type { ApiFormatKey, ThemeKey } from '../types/ui'

export const OFFICIAL_API_BASE_URL: Record<ApiFormatKey, string> = {
  openai: 'https://api.openai.com/v1',
  google: 'https://generativelanguage.googleapis.com/v1beta',
  anthropic: 'https://api.anthropic.com',
}

export const DEFAULT_API_MODELS: Record<ApiFormatKey, string> = {
  openai: 'gpt',
  google: 'gemini',
  anthropic: 'claude',
}

export const DEFAULT_ANTHROPIC_VERSION = '2023-06-01'

export const themes: Array<{ key: ThemeKey; name: string; mode: 'dark' | 'light' }> = [
  { key: 'latte', name: 'Catppuccin Latte', mode: 'light' },
  { key: 'frappe', name: 'Catppuccin Frappe', mode: 'dark' },
  { key: 'macchiato', name: 'Catppuccin Macchiato', mode: 'dark' },
  { key: 'mocha', name: 'Catppuccin Mocha', mode: 'dark' },
]

export const defaultLanguages = [
  { code: 'ZH', name: '中文' },
  { code: 'EN', name: '英语' },
  { code: 'JA', name: '日语' },
  { code: 'KO', name: '韩语' },
  { code: 'RU', name: '俄语' },
  { code: 'DE', name: '德语' },
  { code: 'FR', name: '法语' },
  { code: 'IT', name: '意大利语' },
  { code: 'ES', name: '西班牙语' },
  { code: 'PT', name: '葡萄牙语' },
]
