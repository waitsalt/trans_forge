<script setup lang="ts">
import { EpBrush, EpMoonNight, EpSunny } from 'vue-icons-plus/ep'

defineProps<{
  activeTheme: string
  themes: Array<{ key: string; name: string; mode: 'dark' | 'light' }>
  onUpdateActiveTheme: (value: string) => void
}>()
</script>

<template>
  <div class="page">
    <h2>设置</h2>

    <div class="card">
      <h3 class="theme-title"><EpBrush class="inline-icon" /> 主题配置</h3>
      <p class="theme-subtitle">默认主题为 Catppuccin Mocha（暗色）</p>

      <div class="theme-grid">
        <button
          v-for="theme in themes"
          :key="theme.key"
          class="theme-option"
          :class="{ active: activeTheme === theme.key }"
          @click="onUpdateActiveTheme(theme.key)"
        >
          <span class="theme-option-header">
            <EpMoonNight v-if="theme.mode === 'dark'" class="inline-icon" />
            <EpSunny v-else class="inline-icon" />
            {{ theme.name }}
          </span>
          <span class="theme-option-meta">{{ theme.mode === 'dark' ? 'Dark' : 'Light' }}</span>
        </button>
      </div>
    </div>
  </div>
</template>
