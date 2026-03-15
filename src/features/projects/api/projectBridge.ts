import { api } from '../../../shared/tauri/api'

export const projectBridge = {
  queryProjects: api.queryProjects,
  deleteProjects: api.deleteProjects,
  createProjectProfile: api.createProjectProfile,
  updateProjectProfile: api.updateProjectProfile,
  getProject: api.getProject,
  deleteProject: api.deleteProject,
  getAllProjectRuntimeSnapshots: api.getAllProjectRuntimeSnapshots,
  createProject: api.createProject,
  readInputFiles: api.readInputFiles,
  loadProjectItems: api.loadProjectItems,
  setItems: api.setItems,
  startTranslation: api.startTranslation,
  stopTranslation: api.stopTranslation,
  getProgress: api.getProgress,
  exportFiles: api.exportFiles,
  clearProjectItems: api.clearProjectItems,
}
