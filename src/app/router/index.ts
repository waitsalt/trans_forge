import { createRouter, createWebHashHistory } from 'vue-router'
import ProviderPage from '../../features/providers/views/ProviderPage.vue'
import ProjectPage from '../../features/projects/views/ProjectPage.vue'
import PresetPage from '../../features/presets/views/PresetPage.vue'
import SettingsPage from '../../features/settings/views/SettingsPage.vue'

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: '/', redirect: '/home' },
    { path: '/home', name: 'home', component: ProviderPage },
    { path: '/project', name: 'project', component: ProjectPage },
    { path: '/preset', name: 'preset', component: PresetPage },
    { path: '/settings', name: 'settings', component: SettingsPage },
    { path: '/:pathMatch(.*)*', redirect: '/home' },
  ],
})

export default router
