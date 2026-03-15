<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { providerBridge } from '../api/providerBridge'
import { toErrorMessage } from '../../../shared/utils/core'
import SearchableSelect from '../../../shared/ui/SearchableSelect.vue'
import type { ApiFormatKey } from '../../../shared/types/ui'
import { DEFAULT_API_MODELS } from '../../../shared/constants'

defineProps<{
  flat?: boolean
}>()

const apiFormat = defineModel<ApiFormatKey>('apiFormat', { required: true })
const apiUrl = defineModel<string>('apiUrl', { required: true })
const apiModel = defineModel<string>('apiModel', { required: true })
const anthropicVersion = defineModel<string>('anthropicVersion', { required: true })
const apiTimeout = defineModel<number>('apiTimeout', { required: true })
const requestsPerSecond = defineModel<number>('requestsPerSecond', { required: true })
const requestsPerMinute = defineModel<number>('requestsPerMinute', { required: true })
const requestsPerHour = defineModel<number>('requestsPerHour', { required: true })
const requestsPerDay = defineModel<number>('requestsPerDay', { required: true })
const apiKeysText = defineModel<string>('apiKeysText', { required: true })
const groupStrategy = defineModel<string>('groupStrategy', { required: true })
const maxRetriesPerKey = defineModel<number>('maxRetriesPerKey', { required: true })

const modelOptions = ref<string[]>([])
const modelLoading = ref<boolean>(false)
const modelLoadError = ref<string>('')
const loadedFingerprint = ref<string>('')
const apiFormatOptions: Array<{ value: ApiFormatKey; label: string }> = [
  { value: 'openai', label: 'OpenAI 兼容' },
  { value: 'google', label: 'Google 兼容' },
  { value: 'anthropic', label: 'Anthropic 兼容' },
]
const groupStrategyOptions = [
  { value: 'sequential', label: '顺序' },
  { value: 'random', label: '随机' },
  { value: 'available', label: '可用优先' },
  { value: 'weighted', label: '加权（含速度影响衰减）' },
]

watch(
  () => [apiFormat.value, apiUrl.value, apiKeysText.value],
  () => {
    loadedFingerprint.value = ''
    modelOptions.value = []
  },
)

const isAnthropicFormat = computed(() => apiFormat.value === 'anthropic')
const hasApiKey = computed(() => parseApiKeyEntries(apiKeysText.value).length > 0)
const modelSelectOptions = computed(() => modelOptions.value.map((name) => ({ value: name, label: name })))
const modelPlaceholder = computed(() => DEFAULT_API_MODELS[apiFormat.value] ?? DEFAULT_API_MODELS.openai)
const modelEmptyText = computed(() => {
  if (!hasApiKey.value) {
    return '填写 API Key 后可获取模型列表'
  }
  if (modelLoading.value) {
    return '正在获取模型列表...'
  }
  if (modelLoadError.value) {
    return modelLoadError.value
  }
  return '没有匹配模型，可继续手动输入'
})

function parseApiKeyEntries(text: string): Array<{ key: string; weight: number }> {
  return text
    .split('\n')
    .map((line) => line.trim())
    .filter((line) => line.length > 0)
    .map((line) => {
      const [rawKey, rawWeight] = line.split(',').map((v) => v.trim())
      const weight = Number(rawWeight)
      return {
        key: rawKey,
        weight: Number.isFinite(weight) && weight > 0 ? weight : 1,
      }
    })
    .filter((entry) => entry.key.length > 0)
}

function buildConfigFingerprint(): string {
  const firstKey = parseApiKeyEntries(apiKeysText.value)[0]?.key ?? ''
  return `${apiFormat.value}|${apiUrl.value.trim()}|${firstKey}`
}

async function ensureModelOptionsLoaded() {
  if (!hasApiKey.value) {
    modelLoadError.value = ''
    modelOptions.value = []
    return
  }
  const fingerprint = buildConfigFingerprint()
  if (loadedFingerprint.value === fingerprint && modelOptions.value.length > 0) {
    return
  }

  modelLoading.value = true
  modelLoadError.value = ''
  try {
    const models = await providerBridge.fetchModels({
      config: {
        name: 'model-fetch',
        format:
          apiFormat.value === 'anthropic'
            ? {
                type: 'anthropic',
                anthropic_version: anthropicVersion.value?.trim() || undefined,
              }
            : { type: apiFormat.value },
        api_url: apiUrl.value,
        api_keys: parseApiKeyEntries(apiKeysText.value),
        group_strategy: groupStrategy.value,
        max_retries_per_key: maxRetriesPerKey.value,
        model: apiModel.value.trim() || 'placeholder-model',
        temperature: 0.3,
        timeout: apiTimeout.value,
        requests_per_second: requestsPerSecond.value,
        requests_per_minute: requestsPerMinute.value,
        requests_per_hour: requestsPerHour.value,
        requests_per_day: requestsPerDay.value,
      },
    })
    modelOptions.value = Array.isArray(models) ? models : []
    loadedFingerprint.value = fingerprint
    if (modelOptions.value.length === 0) {
      modelLoadError.value = '未获取到模型列表，可继续手动输入'
    }
  } catch (error) {
    modelOptions.value = []
    modelLoadError.value = `模型列表获取失败，可继续手动输入 (${toErrorMessage(error)})`
  } finally {
    modelLoading.value = false
  }
}

</script>

<template>
  <div :class="{ card: !flat }">
    <h3 v-if="!flat">API 设置</h3>
    <div class="form-group">
      <label>API 格式</label>
      <SearchableSelect
        v-model="apiFormat"
        :options="apiFormatOptions"
        placeholder="选择 API 格式"
        :editable="false"
        :filterable="false"
      />
    </div>

    <div class="form-group" v-if="isAnthropicFormat">
      <label>Anthropic 版本 (anthropic-version)</label>
      <input
        type="text"
        v-model="anthropicVersion"
        placeholder="2023-06-01"
      />
    </div>

    <div class="form-group">
      <label>API URL</label>
      <input
        type="text"
        v-model="apiUrl"
        placeholder="https://api.openai.com/v1"
      />
    </div>

    <div class="form-group">
      <label>API Key 组（每行：key,weight）</label>
      <textarea
        v-model="apiKeysText"
        rows="5"
        placeholder="sk-xxxx,1&#10;sk-yyyy,2"
      ></textarea>
    </div>

    <div class="form-row">
      <div class="form-group">
        <label>组使用策略</label>
        <SearchableSelect
          v-model="groupStrategy"
          :options="groupStrategyOptions"
          placeholder="选择组使用策略"
          :editable="false"
          :filterable="false"
        />
      </div>
      <div class="form-group">
        <label>失败重试次数（每个 key）</label>
        <input
          type="number"
          v-model.number="maxRetriesPerKey"
          min="0"
          max="20"
        />
      </div>
    </div>

    <div class="form-group">
      <label>模型</label>
      <SearchableSelect
        v-model="apiModel"
        :options="modelSelectOptions"
        :placeholder="modelPlaceholder"
        :allow-custom="true"
        :editable="true"
        :filterable="true"
        :empty-text="modelEmptyText"
        :on-open="ensureModelOptionsLoaded"
        :on-query="ensureModelOptionsLoaded"
      />
    </div>

    <div class="form-group">
      <label>超时时间 (秒)</label>
      <input
        type="number"
        v-model.number="apiTimeout"
        min="30"
        max="300"
      />
    </div>

    <div class="form-row">
      <div class="form-group">
        <label>每秒请求次数（0=不限）</label>
        <input
          type="number"
          v-model.number="requestsPerSecond"
          min="0"
        />
      </div>
      <div class="form-group">
        <label>每分钟请求次数（0=不限）</label>
        <input
          type="number"
          v-model.number="requestsPerMinute"
          min="0"
        />
      </div>
    </div>

    <div class="form-row">
      <div class="form-group">
        <label>每小时请求次数（0=不限）</label>
        <input
          type="number"
          v-model.number="requestsPerHour"
          min="0"
        />
      </div>
      <div class="form-group">
        <label>每天请求次数（0=不限）</label>
        <input
          type="number"
          v-model.number="requestsPerDay"
          min="0"
        />
      </div>
    </div>
  </div>
</template>
