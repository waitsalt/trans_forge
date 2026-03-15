export type ProviderFormat =
  | { type: 'openai' }
  | { type: 'google' }
  | { type: 'anthropic'; anthropic_version?: string | null }

export interface Provider {
  name: string
  format: ProviderFormat
  api_url: string
  api_keys: Array<{ key: string; weight: number }>
  group_strategy: string
  max_retries_per_key: number
  model: string
  timeout: number
  requests_per_second: number
  requests_per_minute: number
  requests_per_hour: number
  requests_per_day: number
}
