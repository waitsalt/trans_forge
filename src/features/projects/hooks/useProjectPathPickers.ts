import { open } from '@tauri-apps/plugin-dialog'
import type { Ref } from 'vue'

interface UseProjectPathPickersDeps {
  draftProjectInputPath: Ref<string>
  draftProjectOutputPath: Ref<string>
}

export function useProjectPathPickers(deps: UseProjectPathPickersDeps) {
  const {
    draftProjectInputPath,
    draftProjectOutputPath,
  } = deps

  async function selectInputFolder() {
    const selected = await open({
      directory: false,
      multiple: false,
      title: '选择输入文件或文件夹',
      filters: [
        { name: '支持的文件', extensions: ['txt', 'md', 'srt', 'ass', 'epub', 'xlsx'] },
        { name: '所有文件', extensions: ['*'] },
      ],
    })

    if (selected) {
      const picked = selected as string
      draftProjectInputPath.value = picked
    }
  }

  async function selectInputDirectory() {
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择输入文件夹',
    })

    if (selected) {
      const picked = selected as string
      draftProjectInputPath.value = picked
    }
  }

  async function selectOutputFolder() {
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择输出文件夹',
    })

    if (selected) {
      const picked = selected as string
      draftProjectOutputPath.value = picked
    }
  }

  return {
    selectInputFolder,
    selectInputDirectory,
    selectOutputFolder,
  }
}
