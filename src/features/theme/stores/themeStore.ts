import { computed, ref } from 'vue'
import { themeBridge } from '../api/themeBridge'
import type { ThemeConfig, ThemeInput, ThemePreferences, ThemeState } from '../../../shared/types/theme'
import type { ThemeKey } from '../../../shared/types/ui'

const themes = ref<ThemeConfig[]>([])
const preferences = ref<ThemePreferences | null>(null)
const initialized = ref(false)
const pending = ref(false)

function applyState(state: ThemeState) {
  themes.value = state.themes
  preferences.value = state.preferences
  initialized.value = true
}

async function fetchThemeState() {
  pending.value = true
  try {
    const state = await themeBridge.getThemeState()
    applyState(state)
  } catch (error) {
    console.error('加载主题失败', error)
  } finally {
    pending.value = false
  }
}

async function mutateThemeState(request: () => Promise<ThemeState>) {
  pending.value = true
  try {
    const state = await request()
    applyState(state)
  } catch (error) {
    pending.value = false
    throw error
  }
  pending.value = false
}

export function useThemeStore() {
  async function ensureLoaded() {
    if (!initialized.value && !pending.value) {
      await fetchThemeState()
    }
  }

  async function createTheme(config: ThemeInput) {
    await mutateThemeState(() => themeBridge.createTheme(config))
  }

  async function updateTheme(id: number, config: ThemeInput) {
    await mutateThemeState(() => themeBridge.updateTheme(id, config))
  }

  async function deleteTheme(id: number) {
    await mutateThemeState(() => themeBridge.deleteTheme(id))
  }

  async function restoreDefaults() {
    await mutateThemeState(() => themeBridge.restoreDefaultThemes())
  }

  async function savePreferences(next: ThemePreferences) {
    try {
      const result = await themeBridge.saveThemePreferences(next)
      preferences.value = result
      initialized.value = true
    } catch (error) {
      throw error
    }
  }

  const lightThemes = computed(() => themes.value.filter((theme) => theme.mode === 'light'))
  const darkThemes = computed(() => themes.value.filter((theme) => theme.mode === 'dark'))

  function getThemeByKey(key: ThemeKey) {
    return themes.value.find((theme) => theme.key === key)
  }

  function canDeleteTheme(theme: ThemeConfig) {
    const list = theme.mode === 'light' ? lightThemes.value : darkThemes.value
    return list.length > 1
  }

  return {
    themes,
    preferences,
    initialized,
    pending,
    ensureLoaded,
    createTheme,
    updateTheme,
    deleteTheme,
    restoreDefaults,
    savePreferences,
    getThemeByKey,
    canDeleteTheme,
    lightThemes,
    darkThemes,
  }
}
