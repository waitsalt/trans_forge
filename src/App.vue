<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { Toaster, toast } from 'vue-sonner'
import { RouterView } from 'vue-router'
import AppSidebar from './features/shell/components/AppSidebar.vue'
import './styles/app.css'
import type { ThemePalette } from './shared/types/theme'
import { useThemeStore } from './features/theme/stores/themeStore'

const themeStore = useThemeStore()

const fallbackPalette: ThemePalette = {
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

const systemPrefersDark = ref(false)

const activeThemeKey = computed(() => {
  const prefs = themeStore.preferences.value
  if (!prefs) {
    return 'mocha'
  }
  if (prefs.mode === 'light') {
    return prefs.light_theme_key
  }
  if (prefs.mode === 'system') {
    return systemPrefersDark.value ? prefs.dark_theme_key : prefs.light_theme_key
  }
  return prefs.dark_theme_key
})

const activeTheme = computed(() => {
  return (
    themeStore.getThemeByKey(activeThemeKey.value) ?? {
      id: -1,
      key: activeThemeKey.value,
      name: 'Fallback',
      mode: 'dark',
      palette: fallbackPalette,
      is_builtin: true,
    }
  )
})

const themeStyle = computed(() => {
  const palette = activeTheme.value.palette ?? fallbackPalette
  const scheme = activeTheme.value.mode === 'light' ? 'light' : 'dark'
  return {
    '--bg': palette.base,
    '--bg-elev': palette.mantle,
    '--bg-soft': palette.surface0,
    '--bg-muted': palette.surface2,
    '--text': palette.text,
    '--text-soft': palette.subtext0,
    '--text-dim': palette.overlay1,
    '--border': palette.overlay2,
    '--accent': palette.blue,
    '--ok': palette.green,
    '--warn': palette.yellow,
    '--danger': palette.red,
    'color-scheme': scheme,
  } as Record<string, string>
})

const toasterTheme = computed(() => (activeTheme.value.mode === 'light' ? 'light' : 'dark'))

async function copyToastText(text: string) {
  try {
    await navigator.clipboard.writeText(text)
    toast.success('已复制通知文本', { duration: 1000 })
  } catch (error) {
    toast.error(`复制失败: ${error}`)
  }
}

function handleToastClick(event: MouseEvent) {
  const target = event.target as HTMLElement | null
  if (!target) {
    return
  }
  if (target.closest('[data-sonner-toast] button')) {
    return
  }
  const toastEl = target.closest<HTMLElement>('[data-sonner-toast]')
  if (!toastEl) {
    return
  }
  const text = toastEl.innerText.trim()
  if (!text) {
    return
  }
  copyToastText(text).catch(() => {})
}

let mediaQuery: MediaQueryList | null = null

function handleColorSchemeChange(event: MediaQueryListEvent) {
  systemPrefersDark.value = event.matches
}

onMounted(() => {
  themeStore.ensureLoaded()
  document.addEventListener('click', handleToastClick)
  if (typeof window !== 'undefined' && window.matchMedia) {
    mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
    systemPrefersDark.value = mediaQuery.matches
    mediaQuery.addEventListener('change', handleColorSchemeChange)
  }
})

onBeforeUnmount(() => {
  document.removeEventListener('click', handleToastClick)
  if (mediaQuery) {
    mediaQuery.removeEventListener('change', handleColorSchemeChange)
  }
})
</script>

<template>
  <div class="app" :data-theme="activeTheme.key" :style="themeStyle">
    <Toaster
      :theme="toasterTheme"
      position="top-right"
      :rich-colors="true"
      :close-button="true"
      :visible-toasts="4"
    />
    <AppSidebar />

    <main class="main-content">
      <RouterView />
    </main>
  </div>
</template>
