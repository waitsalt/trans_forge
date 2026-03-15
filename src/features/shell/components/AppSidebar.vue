<script setup lang="ts">
import { EpCollectionTag, EpCpu, EpFolderOpened, EpSetting } from 'vue-icons-plus/ep'
import { RouterLink } from 'vue-router'
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
      <RouterLink to="/settings" class="theme-btn setting-btn" active-class="active">
        <EpSetting class="theme-icon" />
        <span>设置</span>
      </RouterLink>
    </div>
  </aside>
</template>
