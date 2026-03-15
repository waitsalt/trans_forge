import { api } from '../../../shared/tauri/api'
import type { ThemeInput, ThemePreferences, ThemeState } from '../../../shared/types/theme'

export const themeBridge = {
  getThemeState: (): Promise<ThemeState> => api.getThemeState(),
  createTheme: (config: ThemeInput): Promise<ThemeState> => api.createTheme({ config }),
  updateTheme: (id: number, config: ThemeInput): Promise<ThemeState> => api.updateTheme({ id, config }),
  deleteTheme: (id: number): Promise<ThemeState> => api.deleteTheme({ id }),
  restoreDefaultThemes: (): Promise<ThemeState> => api.restoreDefaultThemes(),
  saveThemePreferences: (prefs: ThemePreferences): Promise<ThemePreferences> =>
    api.saveThemePreferences({ prefs }),
}
