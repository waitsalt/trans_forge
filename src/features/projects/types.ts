export type ProjectStatusCode = 'not_started' | 'running' | 'paused' | 'completed'

export interface Project {
  name: string
  source_language: string
  target_language: string
  provider_name: string
  concurrent_limit: number
  prompt?: string | null
  run_status?: ProjectStatusCode
  input_path: string
  output_path: string
  created_at?: string
  updated_at?: string
}

export interface TranslationItem {
  id: string
  file_type: string
  file_path: string
  index: number
  source_text: string
  translated_text: string
  status: string
  error_message: string | null
}

export interface TranslationProgress {
  total: number
  processed: number
  error: number
  is_running: boolean
  current_item: string | null
}

export interface ProjectRuntimeSnapshot {
  name: string
  status: ProjectStatusCode
  total: number
  processed: number
  error: number
}
