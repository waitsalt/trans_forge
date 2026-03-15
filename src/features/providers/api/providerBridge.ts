import { api } from '../../../shared/tauri/api'

export const providerBridge = {
  queryProviders: api.queryProviders,
  getProvider: api.getProvider,
  createProvider: api.createProvider,
  updateProvider: api.updateProvider,
  deleteProvider: api.deleteProvider,
  deleteProviders: api.deleteProviders,
  testProvider: api.testProvider,
  fetchModels: api.fetchModels,
}
