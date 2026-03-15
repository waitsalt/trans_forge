<script setup lang="ts">
import { EpCollectionTag, EpCpu, EpFolderOpened, EpMoonNight, EpSetting, EpSunny } from 'vue-icons-plus/ep'
import { RouterLink } from 'vue-router'

defineProps<{
  isDark: boolean
  onToggleTheme: () => void
}>()
</script>

<template>
  <aside class="sidebar">
    <nav class="nav-menu">
      <RouterLink to="/home" class="nav-item" active-class="active">
        <EpCpu class="nav-icon" />
        <span class="label">模型</span>
      </RouterLink>

      <RouterLink to="/project" class="nav-item" active-class="active">
        <EpFolderOpened class="nav-icon" />
        <span class="label">项目</span>
      </RouterLink>
      <RouterLink to="/preset" class="nav-item" active-class="active">
        <EpCollectionTag class="nav-icon" />
        <span class="label">预设</span>
      </RouterLink>

    </nav>

    <div class="theme-toggle">
      <button class="theme-btn" @click="onToggleTheme">
        <EpSunny v-if="isDark" class="theme-icon" />
        <EpMoonNight v-else class="theme-icon" />
        <span v-if="isDark">切换浅色</span>
        <span v-else>切换暗色</span>
      </button>
      <RouterLink to="/settings" class="theme-btn setting-btn" active-class="active">
        <EpSetting class="theme-icon" />
        <span>设置</span>
      </RouterLink>
    </div>
  </aside>
</template>
