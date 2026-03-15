<script setup lang="ts">
import { watch } from 'vue'
import SearchableSelect from '../../../shared/ui/SearchableSelect.vue'
import { useProjectPageVm } from '../hooks/projectPageContext'

const {
  languages,
  editingProjectName,
  draftProjectName,
  draftInputPath,
  draftOutputPath,
  draftSourceLanguage,
  draftTargetLanguage,
  draftProviderName,
  draftConcurrentLimit,
  draftPrompt,
  draftPromptPresetName,
  promptPresetOptions,
  providerOptions,
  applyPromptPresetToProject,
  selectInputFolder,
  selectInputDirectory,
  selectOutputFolder,
  saveProjectProfile,
  cancelEditProject,
} = useProjectPageVm()

watch(draftPromptPresetName, (value) => {
  applyPromptPresetToProject(value)
})
</script>

<template>
  <div class="card">
    <h3>{{ editingProjectName ? '编辑配置' : '创建配置' }}</h3>
    <div class="form-group">
      <label>配置名称</label>
      <input
        type="text"
        v-model="draftProjectName"
        placeholder="例如：my_project"
      />
    </div>

    <div class="form-row">
      <div class="form-group">
        <label>原文语言</label>
        <SearchableSelect
          v-model="draftSourceLanguage"
          :options="languages.map((v) => ({ value: v.code, label: v.name }))"
          placeholder="选择原文语言"
          :editable="false"
          :filterable="false"
        />
      </div>
      <div class="form-group">
        <label>译文语言</label>
        <SearchableSelect
          v-model="draftTargetLanguage"
          :options="languages.map((v) => ({ value: v.code, label: v.name }))"
          placeholder="选择译文语言"
          :editable="false"
          :filterable="false"
        />
      </div>
    </div>

    <div class="form-group">
      <label>关联 Provider</label>
      <SearchableSelect
        v-model="draftProviderName"
        :options="providerOptions"
        placeholder="选择 Provider"
        :editable="false"
        :filterable="false"
      />
    </div>

    <div class="form-group">
      <label>并发量</label>
      <input
        type="number"
        min="1"
        v-model.number="draftConcurrentLimit"
      />
    </div>

    <div class="form-group">
      <label>Prompt 预设</label>
      <SearchableSelect
        v-model="draftPromptPresetName"
        :options="[{ value: '', label: '不使用预设' }, ...promptPresetOptions]"
        placeholder="选择预设名称"
        :editable="false"
        :filterable="false"
      />
    </div>

    <div class="form-group">
      <label>Prompt（可选）</label>
      <textarea
        class="project-prompt-input"
        v-model="draftPrompt"
        placeholder="可留空使用默认 Prompt。支持占位符：{source}、{target}"
      ></textarea>
      <p class="prompt-placeholder-hint">
        支持占位符 {source}（原文语言代码）、{target}（目标语言代码），由后端在调用接口前自动替换。
      </p>
    </div>

    <div class="form-group">
      <label>输入 (文件或文件夹)</label>
      <div class="input-row">
        <input
          type="text"
          v-model="draftInputPath"
          placeholder="选择要翻译的文件或目录"
        />
        <button @click="selectInputFolder()">选择文件</button>
        <button @click="selectInputDirectory()">选择文件夹</button>
      </div>
    </div>

    <div class="form-group">
      <label>输出文件夹</label>
      <div class="input-row">
        <input
          type="text"
          v-model="draftOutputPath"
          placeholder="翻译结果输出目录"
        />
        <button @click="selectOutputFolder()">浏览</button>
      </div>
    </div>

    <div class="model-actions editor-actions">
      <button class="btn-primary" @click="saveProjectProfile()">确认</button>
      <button @click="cancelEditProject()">取消</button>
    </div>
  </div>
</template>

<style scoped>
.project-prompt-input {
  width: 100%;
  min-height: 120px;
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--bg-soft);
  color: var(--text);
  padding: 10px 12px;
  line-height: 1.5;
  resize: vertical;
  outline: none;
}

.project-prompt-input:focus {
  border-color: var(--accent);
  box-shadow: 0 0 0 1px color-mix(in srgb, var(--accent) 35%, transparent);
}

.prompt-placeholder-hint {
  margin-top: 6px;
  font-size: 12px;
  color: color-mix(in srgb, var(--text) 70%, transparent);
}
</style>
