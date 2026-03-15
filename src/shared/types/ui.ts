export interface Language {
  code: string
  name: string
}

export type ThemeKey = string
export type ThemeKind = 'light' | 'dark'
export type ThemeMode = ThemeKind | 'system'
export type ApiFormatKey = 'openai' | 'google' | 'anthropic'
