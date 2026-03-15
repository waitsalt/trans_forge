<script setup lang="ts">
import ApiSettingsCard from '../components/ApiSettingsCard.vue'
import ListHeaderBar from '../../../shared/ui/ListHeaderBar.vue'
import PaginationBar from '../../../shared/ui/PaginationBar.vue'
import { useProviderPage } from '../hooks/useProviderPage'

const {
  providerSearchKeyword,
  openCreateProvider,
  selectAllVisible,
  selectedCount,
  bulkDeleteSelected,
  showEditor,
  pageSize,
  updateProviderPageSize,
  currentPage,
  updateProviderPage,
  totalPages,
  visiblePages,
  providers,
  toggleSelectConfig,
  openEditProvider,
  deleteProvider,
  runningProviderTestNames,
  testProvider,
  editingProviderName,
  draftProviderName,
  draftApiFormat,
  draftApiUrl,
  draftApiModel,
  draftAnthropicVersion,
  draftApiTimeout,
  draftRequestsPerSecond,
  draftRequestsPerMinute,
  draftRequestsPerHour,
  draftRequestsPerDay,
  draftApiKeysText,
  draftGroupStrategy,
  draftMaxRetriesPerKey,
  saveProvider,
  cancelEditProvider,
} = useProviderPage()
</script>

<template>
  <div class="page">
    <ListHeaderBar
      title="模型"
      :search-keyword="providerSearchKeyword"
      search-placeholder="名称关键词"
      add-label="添加配置"
      :selected-count="selectedCount"
      :on-update-search-keyword="(value) => (providerSearchKeyword = value)"
      :on-add="openCreateProvider"
      :on-select-all="selectAllVisible"
      :on-bulk-delete="bulkDeleteSelected"
    />

    <template v-if="!showEditor">
      <div class="list-toolbar">
        <PaginationBar
          :page-size="pageSize"
          :current-page="currentPage"
          :total-pages="totalPages"
          :visible-pages="visiblePages"
          :on-update-page-size="updateProviderPageSize"
          :on-update-current-page="updateProviderPage"
          :on-first="() => updateProviderPage(0)"
          :on-prev="() => updateProviderPage(Math.max(currentPage - 1, 0))"
          :on-go="updateProviderPage"
          :on-next="() => updateProviderPage(Math.min(currentPage + 1, Math.max(totalPages - 1, 0)))"
          :on-last="() => updateProviderPage(Math.max(totalPages - 1, 0))"
        />
      </div>

      <div v-if="providers.length === 0" class="card empty-state">当前没有相关配置,请添加</div>
      <div v-else class="model-grid">
        <div v-for="item in providers" :key="item.name" class="card model-card">
          <button
            class="card-serial-badge serial-select-btn"
            :class="{ active: item.selected }"
            @click="toggleSelectConfig(item.name)"
          >
            {{ item.serial }}
          </button>
          <div class="model-meta">
            <div><span class="meta-label">名称</span><span class="meta-value">{{ item.name }}</span></div>
            <div><span class="meta-label">类型</span><span class="meta-value">{{ item.formatType }}</span></div>
            <div><span class="meta-label">密钥个数</span><span class="meta-value">{{ item.keyCount }}</span></div>
          </div>
          <div class="model-actions">
            <button
              class="btn btn--ghost"
              :disabled="runningProviderTestNames.includes(item.name)"
              @click="testProvider(item.name)"
            >
              {{ runningProviderTestNames.includes(item.name) ? '测试中...' : '测试' }}
            </button>
            <button class="btn" @click="openEditProvider(item.name)">编辑</button>
            <button class="btn btn--danger" @click="deleteProvider(item.name)">删除</button>
          </div>
        </div>
      </div>
    </template>

    <div v-if="showEditor" class="card">
      <h3>{{ editingProviderName ? '编辑配置' : '创建配置' }}</h3>
      <div class="form-group">
        <label>配置名称</label>
        <input
          type="text"
          v-model="draftProviderName"
          placeholder="例如：openai-main"
        />
      </div>

      <ApiSettingsCard
        :flat="true"
        v-model:api-format="draftApiFormat"
        v-model:api-url="draftApiUrl"
        v-model:api-model="draftApiModel"
        v-model:anthropic-version="draftAnthropicVersion"
        v-model:api-timeout="draftApiTimeout"
        v-model:requests-per-second="draftRequestsPerSecond"
        v-model:requests-per-minute="draftRequestsPerMinute"
        v-model:requests-per-hour="draftRequestsPerHour"
        v-model:requests-per-day="draftRequestsPerDay"
        v-model:api-keys-text="draftApiKeysText"
        v-model:group-strategy="draftGroupStrategy"
        v-model:max-retries-per-key="draftMaxRetriesPerKey"
      />

      <div class="model-actions editor-actions">
        <button class="btn btn--accent" @click="saveProvider()">确认</button>
        <button class="btn btn--ghost" @click="cancelEditProvider()">取消</button>
      </div>
    </div>
  </div>
</template>
