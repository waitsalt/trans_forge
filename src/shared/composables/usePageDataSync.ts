import { onMounted, watch, type Ref, type WatchSource } from 'vue'

interface ToastLike {
  error: (message: string) => void
}

interface WatchTask {
  filterSources: WatchSource<unknown>[]
  page: Ref<number>
  task: () => Promise<void>
  errorPrefix: string
}

interface UsePageDataSyncDeps {
  toast: ToastLike
  initialTasks: Array<{ task: () => Promise<void>; errorPrefix: string }>
  watchTasks: WatchTask[]
}

export function usePageDataSync(deps: UsePageDataSyncDeps) {
  const runLoadTask = async (task: () => Promise<void>, errorPrefix: string) => {
    try {
      await task()
    } catch (error) {
      deps.toast.error(`${errorPrefix}: ${error}`)
    }
  }

  const watchPaginatedList = (watchTask: WatchTask) => {
    watch(watchTask.filterSources, () => {
      watchTask.page.value = 0
      runLoadTask(watchTask.task, watchTask.errorPrefix)
    })
    watch(watchTask.page, () => {
      runLoadTask(watchTask.task, watchTask.errorPrefix)
    })
  }

  for (const watchTask of deps.watchTasks) {
    watchPaginatedList(watchTask)
  }

  onMounted(() => {
    for (const item of deps.initialTasks) {
      runLoadTask(item.task, item.errorPrefix)
    }
  })
}
