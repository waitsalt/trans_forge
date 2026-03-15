import { createApp } from 'vue'
import App from './App.vue'
import router from './app/router'
import 'vue-sonner/style.css'

function detectPlatform(): 'windows' | 'macos' | 'linux' | 'web' {
  const ua = navigator.userAgent.toLowerCase()
  const navWithUAData = navigator as Navigator & { userAgentData?: { platform?: string } }
  const platform = (navWithUAData.userAgentData?.platform || navigator.platform || '').toLowerCase()
  const source = `${platform} ${ua}`
  if (source.includes('win')) {
    return 'windows'
  }
  if (source.includes('mac') || source.includes('darwin')) {
    return 'macos'
  }
  if (source.includes('linux') || source.includes('x11')) {
    return 'linux'
  }
  return 'web'
}

document.documentElement.setAttribute('data-platform', detectPlatform())

createApp(App).use(router).mount('#app')
