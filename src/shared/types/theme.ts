import type { ThemeKey, ThemeKind, ThemeMode } from './ui'

export interface ThemePalette {
  rosewater: string
  flamingo: string
  pink: string
  mauve: string
  red: string
  maroon: string
  peach: string
  yellow: string
  green: string
  teal: string
  sky: string
  sapphire: string
  blue: string
  lavender: string
  text: string
  subtext1: string
  subtext0: string
  overlay2: string
  overlay1: string
  overlay0: string
  surface2: string
  surface1: string
  surface0: string
  base: string
  mantle: string
  crust: string
}

export interface ThemeConfig {
  id: number
  key: ThemeKey
  name: string
  mode: ThemeKind
  palette: ThemePalette
  is_builtin: boolean
}

export interface ThemeInput {
  name: string
  mode: ThemeKind
  palette: ThemePalette
}

export interface ThemePreferences {
  mode: ThemeMode
  light_theme_key: ThemeKey
  dark_theme_key: ThemeKey
}

export interface ThemeState {
  themes: ThemeConfig[]
  preferences: ThemePreferences
}
