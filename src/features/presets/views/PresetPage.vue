<script setup lang="ts">
import FilterDropdown from '../../../shared/ui/FilterDropdown.vue'
import ListHeaderBar from '../../../shared/ui/ListHeaderBar.vue'
import PaginationBar from '../../../shared/ui/PaginationBar.vue'
import SearchableSelect from '../../../shared/ui/SearchableSelect.vue'
import { usePresetPage } from '../hooks/usePresetPage'

const {
  languages,
  promptPresetSearchKeyword,
  selectedCount,
  showPromptPresetEditor,
  openCreatePromptPreset,
  selectAllVisiblePromptPresets,
  bulkDeleteSelected,
  allLanguagesSelected,
  languagesPartiallySelected,
  availableLanguages,
  selectedLanguages,
  handleToggleAllLanguages,
  toggleLanguage,
  normalizedPromptPresetPageSize,
  updatePromptPresetPageSize,
  promptPresetPage,
  updatePromptPresetPage,
  totalPages,
  goToFirstPromptPresetPage,
  goToPrevPromptPresetPage,
  visiblePages,
  goToPromptPresetPage,
  goToNextPromptPresetPage,
  goToLastPromptPresetPage,
  presets,
  toggleSelectPromptPreset,
  openEditPromptPreset,
  deletePreset,
  editingPromptPresetName,
  draftPromptPresetName,
  draftPromptPresetLanguage,
  draftPromptPresetPrompt,
  savePreset,
  cancelEditPromptPreset,
} = usePresetPage()
</script>

<template>
  <div class="page">
    <ListHeaderBar
      title="预设"
      :search-keyword="promptPresetSearchKeyword"
      search-placeholder="名称或内容关键词"
      add-label="添加提示词"
      :selected-count="selectedCount"
      :on-update-search-keyword="(value) => (promptPresetSearchKeyword = value)"
      :on-add="openCreatePromptPreset"
      :on-select-all="selectAllVisiblePromptPresets"
      :on-bulk-delete="bulkDeleteSelected"
    />

    <template v-if="!showPromptPresetEditor">
      <div class="list-toolbar">
        <FilterDropdown
          label="语言"
          :options="availableLanguages"
          :selected-values="selectedLanguages"
          :all-selected="allLanguagesSelected"
          :partially-selected="languagesPartiallySelected"
          :on-toggle-all="handleToggleAllLanguages"
          :on-toggle-value="toggleLanguage"
        />
        <PaginationBar
          :page-size="normalizedPromptPresetPageSize"
          :current-page="promptPresetPage"
          :total-pages="totalPages"
          :visible-pages="visiblePages"
          :on-update-page-size="updatePromptPresetPageSize"
          :on-update-current-page="updatePromptPresetPage"
          :on-first="goToFirstPromptPresetPage"
          :on-prev="goToPrevPromptPresetPage"
          :on-go="goToPromptPresetPage"
          :on-next="goToNextPromptPresetPage"
          :on-last="goToLastPromptPresetPage"
        />
      </div>

      <div v-if="presets.length === 0" class="card empty-state">当前没有相关预设,请添加</div>
      <div v-else class="model-grid">
          <div v-for="item in presets" :key="item.name" class="card model-card">
            <button
              class="card-serial-badge serial-select-btn"
              :class="{ active: item.selected }"
              @click="toggleSelectPromptPreset(item.name)"
            >
              {{ item.serial }}
            </button>
            <div class="model-meta">
              <div><span class="meta-label">名称</span><span class="meta-value">{{ item.name }}</span></div>
              <div><span class="meta-label">语言</span><span class="meta-value">{{ item.language }}</span></div>
              <div class="preset-prompt-row">
                <span class="meta-label">提示词</span>
                <div class="meta-value preset-prompt">{{ item.prompt }}</div>
              </div>
            </div>
            <div class="model-actions">
              <button @click="openEditPromptPreset(item.name)">编辑</button>
              <button class="danger" @click="deletePreset(item.name)">删除</button>
            </div>
          </div>
      </div>
    </template>

    <div v-if="showPromptPresetEditor" class="card">
      <h3>{{ editingPromptPresetName ? '编辑提示词' : '创建提示词' }}</h3>
      <div class="form-group">
        <label>名称</label>
        <input
          type="text"
          v-model="draftPromptPresetName"
          placeholder="例如：zh-polish-v1"
        />
      </div>
      <div class="form-group">
        <label>语言</label>
        <SearchableSelect
          v-model="draftPromptPresetLanguage"
          :options="languages.map((v) => ({ value: v.code, label: v.name }))"
          placeholder="选择语言"
          :editable="false"
          :filterable="false"
        />
      </div>
      <div class="form-group">
        <label>提示词内容</label>
        <textarea
          class="preset-editor-textarea"
          v-model="draftPromptPresetPrompt"
          placeholder="请输入完整提示词内容"
        ></textarea>
        <p class="prompt-placeholder-hint">
          可使用占位符 {source}（原文语言代码）、{target}（目标语言代码），后端会在翻译时自动替换。
        </p>
      </div>

      <div class="model-actions editor-actions">
        <button class="btn-primary" @click="savePreset">确认</button>
        <button @click="cancelEditPromptPreset">取消</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.prompt-placeholder-hint {
  margin-top: 6px;
  font-size: 12px;
  color: color-mix(in srgb, var(--text) 70%, transparent);
}
</style>
