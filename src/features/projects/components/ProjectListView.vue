<script setup lang="ts">
import FilterDropdown from '../../../shared/ui/FilterDropdown.vue'
import PaginationBar from '../../../shared/ui/PaginationBar.vue'
import { useProjectPageVm } from '../hooks/projectPageContext'

const {
  projectConfigs,
  pageSize,
  currentPage,
  totalPages,
  visiblePages,
  availableRunStatuses,
  selectedRunStatuses,
  allRunStatusesSelected,
  runStatusesPartiallySelected,
  handleToggleAllRunStatuses,
  toggleRunStatus,
  updateProjectProfilePageSize,
  updateProjectProfilePage,
  toggleSelectProject,
  startOrPauseProject,
  retryProjectErrors,
  retryProjectAll,
  openProjectDetail,
  exportProject,
  openEditProject,
  deleteProject,
} = useProjectPageVm()
</script>

<template>
  <div class="list-toolbar">
    <FilterDropdown
      label="运行状态"
      :options="availableRunStatuses"
      :selected-values="selectedRunStatuses"
      :all-selected="allRunStatusesSelected"
      :partially-selected="runStatusesPartiallySelected"
      :on-toggle-all="handleToggleAllRunStatuses"
      :on-toggle-value="toggleRunStatus"
    />
    <PaginationBar
      :page-size="pageSize"
      :current-page="currentPage"
      :total-pages="totalPages"
      :visible-pages="visiblePages"
      :on-update-page-size="updateProjectProfilePageSize"
      :on-update-current-page="updateProjectProfilePage"
      :on-first="() => updateProjectProfilePage(0)"
      :on-prev="() => updateProjectProfilePage(Math.max(currentPage - 1, 0))"
      :on-go="updateProjectProfilePage"
      :on-next="() => updateProjectProfilePage(Math.min(currentPage + 1, Math.max(totalPages - 1, 0)))"
      :on-last="() => updateProjectProfilePage(Math.max(totalPages - 1, 0))"
    />
  </div>

  <div v-if="projectConfigs.length === 0" class="card empty-state">当前没有相关配置,请添加</div>
  <div v-else class="model-grid">
    <div v-for="item in projectConfigs" :key="item.name" class="card model-card">
      <button
        class="card-serial-badge serial-select-btn"
        :class="{ active: item.selected }"
        @click="toggleSelectProject(item.name)"
      >
        {{ item.serial }}
      </button>
      <div class="model-meta">
        <div><span class="meta-label">名称</span><span class="meta-value">{{ item.name }}</span></div>
        <div><span class="meta-label">源语言</span><span class="meta-value">{{ item.sourceLanguage }}</span></div>
        <div><span class="meta-label">目标语言</span><span class="meta-value">{{ item.targetLanguage }}</span></div>
        <div><span class="meta-label">并发量</span><span class="meta-value">{{ item.concurrentLimit }}</span></div>
        <div><span class="meta-label">运行状态</span><span class="meta-value">{{ item.runStatusLabel }}</span></div>
        <div class="progress-row">
          <span class="meta-label">进度</span>
          <div class="progress-block">
            <div class="progress-track">
              <div
                class="progress-done"
                :style="{ width: `${item.progressTotal > 0 ? (item.progressProcessed / item.progressTotal) * 100 : 0}%` }"
              ></div>
              <div
                class="progress-error"
                :style="{ width: `${item.progressTotal > 0 ? (item.progressError / item.progressTotal) * 100 : 0}%` }"
              ></div>
            </div>
            <div class="progress-summary meta-value">
              <span class="progress-active">进行 {{ item.runningCount }}</span>
              <span>完成 {{ item.progressProcessed }} / 错误 {{ item.progressError }} / 未完成 {{ item.progressPending }}</span>
            </div>
          </div>
        </div>
        <div v-if="item.exporting" class="progress-row export-row">
          <span class="meta-label">导出进度</span>
          <div class="progress-block">
            <div class="progress-track export-track">
              <div class="progress-exporting"></div>
            </div>
          </div>
        </div>
      </div>
      <div class="model-actions">
        <button v-if="item.actionLabel !== '完结'" @click="startOrPauseProject(item.name)">
          {{ item.actionLabel }}
        </button>
        <div class="retry-dropdown">
          <button>重试</button>
          <div class="retry-dropdown-menu">
            <button @click="retryProjectErrors(item.name)">错误重试</button>
            <button @click="retryProjectAll(item.name)">全部重试</button>
          </div>
        </div>
        <button @click="openProjectDetail(item.name)">详情</button>
        <button @click="exportProject(item.name)">导出</button>
        <button @click="openEditProject(item.name)">编辑</button>
        <button class="danger" @click="deleteProject(item.name)">删除</button>
      </div>
    </div>
  </div>
</template>
