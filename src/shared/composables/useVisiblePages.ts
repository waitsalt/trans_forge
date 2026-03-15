import { computed, type Ref } from 'vue'

export function useVisiblePages(
  currentPage: Ref<number>,
  totalPages: Ref<number>,
  radius = 2,
) {
  return computed(() => {
    const total = totalPages.value
    if (total <= 0) {
      return []
    }
    const start = Math.max(0, currentPage.value - radius)
    const end = Math.min(total - 1, currentPage.value + radius)
    const pages: number[] = []
    for (let page = start; page <= end; page += 1) {
      pages.push(page)
    }
    return pages
  })
}
