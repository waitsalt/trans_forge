<script setup lang="ts">
import { computed, reactive, ref } from 'vue'
import { EpBrush, EpDelete, EpEdit, EpMonitor, EpMoonNight, EpSunny } from 'vue-icons-plus/ep'
import { toast } from 'vue-sonner'
import type { ThemeConfig, ThemeInput, ThemePalette } from '../../../shared/types/theme'
import type { ThemeKey, ThemeKind, ThemeMode } from '../../../shared/types/ui'
import { useThemeStore } from '../../theme/stores/themeStore'
import SearchableSelect from '../../../shared/ui/SearchableSelect.vue'

const themeStore = useThemeStore()
themeStore.ensureLoaded().catch(() => {})

const preferences = computed(() => themeStore.preferences.value)
const lightThemeOptions = themeStore.lightThemes
const darkThemeOptions = themeStore.darkThemes

const showThemeEditor = ref(false)
const editingTheme = ref<ThemeConfig | null>(null)

const defaultPalette: ThemePalette = {
  rosewater: '#f5e0dc',
  flamingo: '#f2cdcd',
  pink: '#f5c2e7',
  mauve: '#cba6f7',
  red: '#f38ba8',
  maroon: '#eba0ac',
  peach: '#fab387',
  yellow: '#f9e2af',
  green: '#a6e3a1',
  teal: '#94e2d5',
  sky: '#89dceb',
  sapphire: '#74c7ec',
  blue: '#89b4fa',
  lavender: '#b4befe',
  text: '#cdd6f4',
  subtext1: '#bac2de',
  subtext0: '#a6adc8',
  overlay2: '#9399b2',
  overlay1: '#7f849c',
  overlay0: '#6c7086',
  surface2: '#585b70',
  surface1: '#45475a',
  surface0: '#313244',
  base: '#1e1e2e',
  mantle: '#181825',
  crust: '#11111b',
}

const themeDraft = reactive<ThemeInput>({
  name: '',
  mode: 'dark',
  palette: { ...defaultPalette },
})

const paletteFields: Array<{ key: keyof ThemePalette; label: string }> = [
  { key: 'rosewater', label: '柔粉（Rosewater）' },
  { key: 'flamingo', label: '裸粉（Flamingo）' },
  { key: 'pink', label: '粉红（Pink）' },
  { key: 'mauve', label: '丁香紫（Mauve）' },
  { key: 'red', label: '正红（Red）' },
  { key: 'maroon', label: '酒红（Maroon）' },
  { key: 'peach', label: '蜜桃橙（Peach）' },
  { key: 'yellow', label: '琥珀黄（Yellow）' },
  { key: 'green', label: '青柠绿（Green）' },
  { key: 'teal', label: '水鸭青（Teal）' },
  { key: 'sky', label: '浅蓝（Sky）' },
  { key: 'sapphire', label: '蓝晶（Sapphire）' },
  { key: 'blue', label: '亮蓝（Blue）' },
  { key: 'lavender', label: '薰衣草（Lavender）' },
  { key: 'text', label: '主文本（Text）' },
  { key: 'subtext1', label: '辅助文本（Subtext1）' },
  { key: 'subtext0', label: '淡文本（Subtext0）' },
  { key: 'overlay2', label: '覆层深色（Overlay2）' },
  { key: 'overlay1', label: '覆层浅色（Overlay1）' },
  { key: 'overlay0', label: '覆层极浅（Overlay0）' },
  { key: 'surface2', label: '界面深色（Surface2）' },
  { key: 'surface1', label: '界面浅色（Surface1）' },
  { key: 'surface0', label: '界面底色（Surface0）' },
  { key: 'base', label: '基础底色（Base）' },
  { key: 'mantle', label: '背景层（Mantle）' },
  { key: 'crust', label: '最底层（Crust）' },
]

function resetThemeDraft(mode: ThemeKind, source?: ThemeConfig) {
  themeDraft.name = source?.name ?? ''
  themeDraft.mode = source?.mode ?? mode
  themeDraft.palette = source ? { ...source.palette } : { ...defaultPalette }
}

function openCreateTheme(mode: ThemeKind) {
  editingTheme.value = null
  resetThemeDraft(mode)
  showThemeEditor.value = true
}

function openEditTheme(theme: ThemeConfig) {
  editingTheme.value = theme
  resetThemeDraft(theme.mode, theme)
  showThemeEditor.value = true
}

function closeThemeEditor() {
  showThemeEditor.value = false
}

async function submitTheme() {
  const payload: ThemeInput = {
    name: themeDraft.name.trim(),
    mode: themeDraft.mode,
    palette: { ...themeDraft.palette },
  }
  try {
    if (editingTheme.value) {
      await themeStore.updateTheme(editingTheme.value.id, payload)
      toast.success('主题已更新')
    } else {
      await themeStore.createTheme(payload)
      toast.success('主题已创建')
    }
    showThemeEditor.value = false
  } catch (error) {
    toast.error(`保存主题失败: ${error}`)
  }
}

async function handleDeleteTheme(theme: ThemeConfig) {
  if (!themeStore.canDeleteTheme(theme)) {
    toast.error('每种模式至少需要一个主题')
    return
  }
  const confirmed = window.confirm(`确认删除主题「${theme.name}」吗？`)
  if (!confirmed) return
  try {
    await themeStore.deleteTheme(theme.id)
    toast.success('主题已删除')
  } catch (error) {
    toast.error(`删除主题失败: ${error}`)
  }
}

async function handleRestoreDefaults() {
  const confirmed = window.confirm('确定要恢复默认主题吗？这将删除所有自定义主题。')
  if (!confirmed) return
  try {
    await themeStore.restoreDefaults()
    toast.success('已恢复默认主题')
  } catch (error) {
    toast.error(`恢复默认失败: ${error}`)
  }
}

async function updateThemeMode(mode: ThemeMode) {
  if (!preferences.value) return
  try {
    await themeStore.savePreferences({ ...preferences.value, mode })
  } catch (error) {
    toast.error(`切换主题模式失败: ${error}`)
  }
}

async function updateLightTheme(key: ThemeKey) {
  if (!preferences.value || preferences.value.light_theme_key === key) return
  try {
    await themeStore.savePreferences({ ...preferences.value, light_theme_key: key })
  } catch (error) {
    toast.error(`更新浅色主题失败: ${error}`)
  }
}

async function updateDarkTheme(key: ThemeKey) {
  if (!preferences.value || preferences.value.dark_theme_key === key) return
  try {
    await themeStore.savePreferences({ ...preferences.value, dark_theme_key: key })
  } catch (error) {
    toast.error(`更新深色主题失败: ${error}`)
  }
}

const isLoading = computed(() => themeStore.pending.value)
</script>

<template>
  <div class="page">
    <h2>设置</h2>

    <div class="card">
      <h3 class="theme-title"><EpBrush class="inline-icon" /> 主题配置</h3>
      <p class="theme-subtitle">选择主题模式，并为浅色 / 深色模式指定默认主题。</p>

      <div class="theme-mode-grid">
        <button class="theme-mode-btn" :class="{ active: preferences?.mode === 'light' }" @click="updateThemeMode('light')">
          <EpSunny class="inline-icon" />
          <div>
            <div class="theme-mode-label">浅色主题</div>
            <div class="theme-mode-desc">始终使用浅色主题</div>
          </div>
        </button>
        <button class="theme-mode-btn" :class="{ active: preferences?.mode === 'dark' }" @click="updateThemeMode('dark')">
          <EpMoonNight class="inline-icon" />
          <div>
            <div class="theme-mode-label">深色主题</div>
            <div class="theme-mode-desc">始终使用深色主题</div>
          </div>
        </button>
        <button class="theme-mode-btn" :class="{ active: preferences?.mode === 'system' }" @click="updateThemeMode('system')">
          <EpMonitor class="inline-icon" />
          <div>
            <div class="theme-mode-label">跟随系统</div>
            <div class="theme-mode-desc">根据系统深浅色自动切换</div>
          </div>
        </button>
      </div>

      <div class="theme-actions">
        <button class="btn btn--accent" :disabled="isLoading" @click="openCreateTheme('light')">新增浅色主题</button>
        <button class="btn btn--accent" :disabled="isLoading" @click="openCreateTheme('dark')">新增深色主题</button>
        <button class="btn btn--danger" :disabled="isLoading" @click="handleRestoreDefaults">恢复默认</button>
      </div>

      <div class="theme-columns">
        <section class="theme-column">
          <h4>浅色主题</h4>
          <p>设置浅色模式下默认使用的主题。</p>
          <div class="theme-grid theme-option-list">
            <button
              v-for="theme in lightThemeOptions"
              :key="theme.key"
              class="theme-option"
              :class="{ active: preferences?.light_theme_key === theme.key }"
              @click="updateLightTheme(theme.key)"
            >
              <div class="theme-option-header">
                <EpSunny class="inline-icon" />
                <span class="theme-option-name">{{ theme.name }}</span>
              </div>
              <div class="theme-card-footer">
                <span class="theme-option-meta">浅色</span>
                <span class="theme-card-actions" @click.stop>
                  <button
                    type="button"
                    class="btn btn--ghost btn--icon"
                    :title="`编辑 ${theme.name}`"
                    @click="openEditTheme(theme)"
                  >
                    <EpEdit class="inline-icon" />
                  </button>
                  <button
                    type="button"
                    class="btn btn--danger btn--icon"
                    :disabled="!themeStore.canDeleteTheme(theme) || isLoading"
                    :title="`删除 ${theme.name}`"
                    @click="handleDeleteTheme(theme)"
                  >
                    <EpDelete class="inline-icon" />
                  </button>
                </span>
              </div>
            </button>
          </div>
        </section>

        <section class="theme-column">
          <h4>深色主题</h4>
          <p>设置深色模式下默认使用的主题。</p>
          <div class="theme-grid theme-option-list">
            <button
              v-for="theme in darkThemeOptions"
              :key="theme.key"
              class="theme-option"
              :class="{ active: preferences?.dark_theme_key === theme.key }"
              @click="updateDarkTheme(theme.key)"
            >
              <div class="theme-option-header">
                <EpMoonNight class="inline-icon" />
                <span class="theme-option-name">{{ theme.name }}</span>
              </div>
              <div class="theme-card-footer">
                <span class="theme-option-meta">深色</span>
                <span class="theme-card-actions" @click.stop>
                  <button
                    type="button"
                    class="btn btn--ghost btn--icon"
                    :title="`编辑 ${theme.name}`"
                    @click="openEditTheme(theme)"
                  >
                    <EpEdit class="inline-icon" />
                  </button>
                  <button
                    type="button"
                    class="btn btn--danger btn--icon"
                    :disabled="!themeStore.canDeleteTheme(theme) || isLoading"
                    :title="`删除 ${theme.name}`"
                    @click="handleDeleteTheme(theme)"
                  >
                    <EpDelete class="inline-icon" />
                  </button>
                </span>
              </div>
            </button>
          </div>
        </section>
      </div>
    </div>

    <div v-if="showThemeEditor" class="theme-editor-overlay" @click.self="closeThemeEditor">
      <div class="card theme-editor">
        <h3>{{ editingTheme ? '编辑主题' : '创建主题' }}</h3>
        <div class="form-group">
          <label>主题名称</label>
          <input type="text" v-model="themeDraft.name" placeholder="例如：my-theme" />
        </div>
        <div class="form-group">
          <label>主题类型</label>
          <SearchableSelect
            v-model="themeDraft.mode"
            :options="[
              { value: 'light', label: '浅色' },
              { value: 'dark', label: '深色' },
            ]"
            :editable="false"
            :filterable="false"
          />
        </div>
        <div class="palette-grid">
          <label v-for="field in paletteFields" :key="field.key">
            <span>{{ field.label }}</span>
            <div class="color-inputs">
              <input type="color" v-model="themeDraft.palette[field.key]" />
              <input type="text" v-model="themeDraft.palette[field.key]" />
            </div>
          </label>
        </div>
        <div class="model-actions editor-actions">
          <button class="btn btn--accent" :disabled="isLoading" @click="submitTheme">保存</button>
          <button class="btn btn--ghost" @click="closeThemeEditor">取消</button>
        </div>
      </div>
    </div>
  </div>
</template>
