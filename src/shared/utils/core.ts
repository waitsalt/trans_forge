export function toNonNegativeInt(value: number, fallback: number): number {
  if (!Number.isFinite(value)) {
    return fallback
  }
  return Math.max(0, Math.floor(value))
}

export function toPositiveInt(value: number, fallback: number): number {
  if (!Number.isFinite(value) || value <= 0) {
    return fallback
  }
  return Math.floor(value)
}

export function toErrorMessage(error: unknown): string {
  if (error instanceof Error && error.message) {
    return error.message
  }
  return String(error)
}
