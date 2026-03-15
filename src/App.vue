<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { Toaster, toast } from 'vue-sonner'
import { RouterView } from 'vue-router'
import AppSidebar from './features/shell/components/AppSidebar.vue'
import { themes } from './shared/constants'
import './styles/app.css'
import type { ThemeKey } from './shared/types/ui'

const activeTheme = ref<ThemeKey>('mocha')
const isDark = computed(() => activeTheme.value !== 'latte')
const toasterTheme = computed(() => (isDark.value ? 'dark' : 'light'))

function toggleTheme() {
  activeTheme.value = activeTheme.value === 'latte' ? 'mocha' : 'latte'
}

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

onMounted(() => {
  document.addEventListener('click', handleToastClick)
})

onBeforeUnmount(() => {
  document.removeEventListener('click', handleToastClick)
})
</script>

<template>
  <div class="app" :data-theme="activeTheme">
    <Toaster
      :theme="toasterTheme"
      position="top-right"
      :rich-colors="true"
      :close-button="true"
      :visible-toasts="4"
    />
    <AppSidebar
      :is-dark="isDark"
      :on-toggle-theme="toggleTheme"
    />

    <main class="main-content">
      <RouterView v-slot="{ Component, route: viewRoute }">
        <component
          :is="Component"
          v-if="viewRoute.name === 'settings'"
          :active-theme="activeTheme"
          :themes="themes"
          :on-update-active-theme="(value: string) => (activeTheme = value as ThemeKey)"
        />
        <component :is="Component" v-else />
      </RouterView>
    </main>
  </div>
</template>
