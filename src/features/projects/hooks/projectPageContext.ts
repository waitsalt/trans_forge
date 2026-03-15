import { inject, type InjectionKey } from 'vue'
import { useProjectPage } from './useProjectPage'

export type ProjectPageVm = ReturnType<typeof useProjectPage>

export const projectPageVmKey: InjectionKey<ProjectPageVm> = Symbol('projectPageVm')

export function useProjectPageVm(): ProjectPageVm {
  const vm = inject(projectPageVmKey)
  if (!vm) {
    throw new Error('ProjectPageVm not provided')
  }
  return vm
}
