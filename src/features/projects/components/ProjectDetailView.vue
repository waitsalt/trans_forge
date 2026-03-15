<script setup lang="ts">
import { EpClose } from 'vue-icons-plus/ep'
import PaginationBar from '../../../shared/ui/PaginationBar.vue'
import { useProjectPageVm } from '../hooks/projectPageContext'

const {
  detailProjectName,
  detailPrompt,
  closeProjectDetail,
  exportProject,
  clearProjectCache,
  availableStatuses,
  selectedStatuses,
  mapDetailStatusLabel,
  normalizedDetailPageSize,
  updateDetailPageSize,
  detailPage,
  updateDetailPage,
  detailTotalPages,
  goToFirstDetailPage,
  goPrevDetailPage,
  visibleDetailPages,
  goToDetailPage,
  goNextDetailPage,
  goToLastDetailPage,
  visibleDetailRows,
  openTextEditor,
  isDetailItemRunning,
  triggerRunDetailItem,
  mapDetailActionLabel,
  showTextEditor,
  editingField,
  editingText,
  closeTextEditor,
  saveTextEditor,
  toggleDetailStatus,
} = useProjectPageVm()
</script>

<template>
  <div class="section-header">
    <h2>项目详情</h2>
    <div class="header-actions">
      <button @click="closeProjectDetail()">返回</button>
    </div>
  </div>

  <div class="card detail-panel">
    <div class="detail-header">
      <h3>{{ detailProjectName }}</h3>
      <div class="detail-header-actions">
        <button class="detail-head-btn" @click="exportProject(detailProjectName)">导出</button>
        <button class="detail-head-btn danger-outline" @click="clearProjectCache(detailProjectName)">
          清除缓存
        </button>
      </div>
    </div>

    <div class="form-group">
      <label>最终 Prompt（不含源文本）</label>
      <textarea class="detail-prompt" :value="detailPrompt" readonly></textarea>
    </div>

    <div class="detail-toolbar">
      <div class="status-filters">
        <label v-for="status in availableStatuses" :key="status" class="status-chip">
          <input
            type="checkbox"
            :checked="selectedStatuses.includes(status)"
            @change="toggleDetailStatus(status)"
          />
          <span>{{ mapDetailStatusLabel(status) }}</span>
        </label>
      </div>
      <div class="pagination-wrap detail-pagination-wrap">
        <PaginationBar
          :page-size="normalizedDetailPageSize"
          :current-page="detailPage"
          :total-pages="detailTotalPages"
          :visible-pages="visibleDetailPages"
          :on-update-page-size="updateDetailPageSize"
          :on-update-current-page="updateDetailPage"
          :on-first="goToFirstDetailPage"
          :on-prev="goPrevDetailPage"
          :on-go="goToDetailPage"
          :on-next="goNextDetailPage"
          :on-last="goToLastDetailPage"
        />
      </div>
    </div>

    <div class="detail-table-wrap">
      <table class="detail-table">
        <thead>
          <tr>
            <th>序号</th>
            <th>状态</th>
            <th>文件</th>
            <th>源文本</th>
            <th>译文</th>
            <th>错误</th>
            <th>操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="row in visibleDetailRows" :key="row.item.id">
            <td>{{ row.serial }}</td>
            <td>{{ mapDetailStatusLabel(row.item.normalizedStatus) }}</td>
            <td :title="row.item.file_path">{{ row.item.file_path }}</td>
            <td>
              <div
                class="text-cell"
                :title="row.item.source_text"
                @dblclick="openTextEditor(row.item, 'source_text')"
              >
                {{ row.item.source_text }}
              </div>
            </td>
            <td>
              <div
                class="text-cell"
                :title="row.item.translated_text"
                @dblclick="openTextEditor(row.item, 'translated_text')"
              >
                {{ row.item.translated_text }}
              </div>
            </td>
            <td :title="row.item.error_message ?? ''">{{ row.item.error_message ?? '' }}</td>
            <td>
              <div class="row-actions">
                <button
                  class="row-action-btn"
                  :disabled="isDetailItemRunning(row.item.id)"
                  @click="triggerRunDetailItem(detailProjectName, row.item.id, row.item.normalizedStatus)"
                >
                  {{ isDetailItemRunning(row.item.id) ? '执行中...' : mapDetailActionLabel(row.item.normalizedStatus) }}
                </button>
              </div>
            </td>
          </tr>
          <tr v-if="visibleDetailRows.length === 0">
            <td colspan="7" class="empty-row">无匹配条目</td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>

  <div v-if="showTextEditor" class="editor-mask" @click.self="closeTextEditor">
    <div class="editor-panel card">
      <div class="detail-header">
        <h3>{{ editingField === 'source_text' ? '编辑源文本' : '编辑译文' }}</h3>
        <button class="detail-editor-close" aria-label="关闭" @click="closeTextEditor">
          <EpClose />
        </button>
      </div>
      <textarea v-model="editingText" class="editor-textarea"></textarea>
      <div class="model-actions editor-actions">
        <button class="detail-action-btn detail-action-primary" @click="saveTextEditor">保存</button>
        <button class="detail-action-btn" @click="closeTextEditor">取消</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.detail-panel {
  width: 100%;
}

.detail-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 12px;
}

.detail-header h3 {
  margin: 0;
}

.detail-header-actions {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  margin-left: auto;
}

.detail-head-btn {
  border: 1px solid var(--border);
  background: var(--bg-muted);
  color: var(--text);
  border-radius: 10px;
  height: 36px;
  padding: 0 12px;
  font-weight: 600;
}

.detail-head-btn:hover {
  border-color: var(--accent);
  background: var(--bg-soft);
}

.danger-outline {
  border-color: var(--danger);
  color: var(--danger);
  background: var(--bg-elev);
}

.danger-outline:hover {
  border-color: var(--danger);
  background: var(--bg-soft);
}

.detail-editor-close {
  width: 34px;
  height: 34px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--bg-muted);
  color: var(--text-soft);
  cursor: pointer;
}

.detail-editor-close:hover {
  border-color: var(--accent);
  background: var(--bg-soft);
  color: var(--text);
}

.detail-action-btn {
  height: 38px;
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--bg-muted);
  color: var(--text);
  padding: 0 14px;
  font-weight: 600;
}

.detail-action-btn:hover {
  border-color: var(--accent);
  background: var(--bg-soft);
}

.detail-action-primary {
  background: var(--accent);
  border-color: var(--accent);
  color: #fff;
}

.detail-action-primary:hover {
  filter: brightness(0.98);
}

.detail-prompt {
  width: 100%;
  min-height: 120px;
  resize: vertical;
}

.detail-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  flex-wrap: nowrap;
  margin: 8px 0 12px;
}

.status-filters {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  flex: 1 1 auto;
}

.detail-pagination-wrap {
  margin-bottom: 0;
  flex: 0 0 auto;
}

.status-chip {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 4px 8px;
  border: 1px solid var(--border);
  border-radius: 999px;
}

.detail-table-wrap {
  overflow: auto;
  border: 1px solid var(--border);
  border-radius: 10px;
}

.detail-table {
  width: 100%;
  border-collapse: collapse;
  table-layout: fixed;
  font-size: 13px;
}

.detail-table th,
.detail-table td {
  border-bottom: 1px solid var(--border);
  padding: 8px;
  text-align: left;
  vertical-align: top;
  word-break: break-word;
}

.detail-table th:nth-child(1),
.detail-table td:nth-child(1) {
  width: 72px;
}

.detail-table th:nth-child(2),
.detail-table td:nth-child(2) {
  width: 88px;
}

.detail-table th:nth-child(7),
.detail-table td:nth-child(7) {
  width: 128px;
}

.text-cell {
  display: block;
  box-sizing: border-box;
  max-height: 112px;
  overflow-y: auto;
  white-space: pre-wrap;
  line-height: 1.6;
  padding-top: 4px;
  padding-bottom: 2px;
  overscroll-behavior: contain;
  padding-right: 4px;
}

.editor-mask {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.45);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 90;
  padding: 24px;
}

.editor-panel {
  width: min(860px, 95vw);
}

.editor-textarea {
  width: 100%;
  min-height: 320px;
  resize: vertical;
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--bg-soft);
  color: var(--text);
  padding: 12px;
  line-height: 1.6;
  outline: none;
  margin-bottom: 14px;
}

.editor-textarea:focus {
  border-color: var(--accent);
  box-shadow: 0 0 0 1px color-mix(in srgb, var(--accent) 35%, transparent);
}

.row-actions {
  display: flex;
  gap: 8px;
}

.row-action-btn {
  padding: 4px 10px;
  border: 1px solid var(--border);
  background: var(--bg-muted);
  color: var(--text);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.18s ease;
}

.row-action-btn:hover {
  border-color: var(--accent);
  background: var(--bg-soft);
}

.row-action-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.empty-row {
  text-align: center;
  color: var(--text-dim);
}
</style>
