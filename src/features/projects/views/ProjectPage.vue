<script setup lang="ts">
import { provide } from 'vue'
import ListHeaderBar from '../../../shared/ui/ListHeaderBar.vue'
import ProjectDetailView from '../components/ProjectDetailView.vue'
import ProjectEditorView from '../components/ProjectEditorView.vue'
import ProjectListView from '../components/ProjectListView.vue'
import { projectPageVmKey } from '../hooks/projectPageContext'
import { useProjectPage } from '../hooks/useProjectPage'

const vm = useProjectPage()
provide(projectPageVmKey, vm)

const {
  showDetail,
  showEditor,
  searchKeyword,
  selectedCount,
  openCreateProject,
  selectAllVisibleProjects,
  bulkDeleteSelected,
} = vm
</script>

<template>
  <div class="page">
    <ProjectDetailView v-if="showDetail" />
    <template v-else>
      <ListHeaderBar
        title="项目"
        :search-keyword="searchKeyword"
        search-placeholder="名称关键词"
        add-label="添加配置"
        :selected-count="selectedCount"
        :on-update-search-keyword="(value) => (searchKeyword = value)"
        :on-add="openCreateProject"
        :on-select-all="selectAllVisibleProjects"
        :on-bulk-delete="bulkDeleteSelected"
      />

      <ProjectListView v-if="!showEditor" />
      <ProjectEditorView v-else />
    </template>
  </div>
</template>
