<template>
  <div
    class="app"
    @touchstart="handleTouchStart"
    @touchend="handleTouchEnd"
  >
    <h1 class="main-title">📱 Coupon & Receipt Manager</h1>

    <div class="tabs">
      <button
        v-for="tab in tabs"
        :key="tab.id"
        :class="{ active: activeTab === tab.id }"
        @click="activeTab = tab.id"
      >
        {{ tab.name }}
      </button>
    </div>

    <div class="tab-content">
      <ShopsView v-if="activeTab === 'shops'" :selected-shop-id="selectedShopId" />
      <CouponsView v-if="activeTab === 'coupons'" />
      <ReceiptsView v-if="activeTab === 'receipts'" @go-to-shop="openShopFromAnyView" />
      <SettingsView
        v-if="activeTab === 'settings'"
        :settings="settings"
        @update-settings="updateSettings"
      />
    </div>
  </div>
</template>

<script setup>
import { reactive, ref, watch } from 'vue'
import ShopsView from './components/ShopsView.vue'
import CouponsView from './components/CouponsView.vue'
import ReceiptsView from './components/ReceiptsView.vue'
import SettingsView from './components/SettingsView.vue'

const settingsStorageKey = 'coupon-app-settings'

const defaultSettings = {
  mode: 'light',
  theme: 'ocean',
  fontFamily: 'system',
  fontSize: 16,
}

const modePresets = {
  light: {
    bg: '#eef4ff',
    surface: '#ffffff',
    surfaceAlt: '#f7f9ff',
    text: '#10203a',
    muted: '#5f6b85',
    border: '#d7e1ff',
  },
  dark: {
    bg: '#070b14',
    surface: '#0d1422',
    surfaceAlt: '#111a2b',
    text: '#f4f7fb',
    muted: '#9aa7bc',
    border: '#22304a',
  },
}

const themePresets = {
  ocean: {
    accent: '#4f7cff',
    accentStrong: '#315ee5',
    headerStart: '#4f7cff',
    headerEnd: '#7f5af0',
  },
  ember: {
    accent: '#f16b49',
    accentStrong: '#d8502d',
    headerStart: '#f16b49',
    headerEnd: '#ff9b71',
  },
  forest: {
    accent: '#2f9e67',
    accentStrong: '#247d51',
    headerStart: '#2f9e67',
    headerEnd: '#60b67e',
  },
  graphite: {
    accent: '#5b6470',
    accentStrong: '#424b57',
    headerStart: '#4b5563',
    headerEnd: '#6b7280',
  },
}

const fontFamilies = {
  system: 'Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif',
  rounded: '"Trebuchet MS", "Segoe UI", sans-serif',
  serif: 'Georgia, "Times New Roman", serif',
  mono: 'ui-monospace, SFMono-Regular, Menlo, Consolas, monospace',
}

const loadSettings = () => {
  try {
    const stored = JSON.parse(localStorage.getItem(settingsStorageKey) ?? '{}')
    const inferredMode = stored.mode === 'dark' || stored.theme === 'midnight' ? 'dark' : 'light'
    const inferredTheme = stored.theme in themePresets ? stored.theme : defaultSettings.theme
    return {
      mode: inferredMode,
      theme: inferredTheme,
      fontFamily: stored.fontFamily in fontFamilies ? stored.fontFamily : defaultSettings.fontFamily,
      fontSize: Number.isFinite(stored.fontSize) ? Math.min(22, Math.max(14, stored.fontSize)) : defaultSettings.fontSize,
    }
  } catch {
    return { ...defaultSettings }
  }
}

const settings = reactive(loadSettings())

const activeTab = ref('shops')
const selectedShopId = ref(null)
const tabs = [
  { id: 'shops', name: '🏪 Shops' },
  { id: 'coupons', name: '🎫 Coupons' },
  { id: 'receipts', name: '🧾 Receipts' },
  { id: 'settings', name: '⚙️ Settings' },
]

const openShopFromAnyView = (shopId) => {
  if (!shopId) return

  selectedShopId.value = shopId
  activeTab.value = 'shops'
}

const updateSettings = (nextSettings) => {
  Object.assign(settings, nextSettings)
}

const applySettings = () => {
  const root = document.documentElement.style
  const modePalette = modePresets[settings.mode] ?? modePresets.light
  const palette = themePresets[settings.theme] ?? themePresets.ocean

  document.body.dataset.appMode = settings.mode

  root.setProperty('--app-bg', modePalette.bg)
  root.setProperty('--app-surface', modePalette.surface)
  root.setProperty('--app-surface-alt', modePalette.surfaceAlt)
  root.setProperty('--app-text', modePalette.text)
  root.setProperty('--app-muted', modePalette.muted)
  root.setProperty('--app-border', modePalette.border)
  root.setProperty('--app-accent', palette.accent)
  root.setProperty('--app-accent-strong', palette.accentStrong)
  root.setProperty('--app-header-start', palette.headerStart)
  root.setProperty('--app-header-end', palette.headerEnd)
  root.setProperty('--app-font-size', `${settings.fontSize}px`)
  root.setProperty('--app-font-family', fontFamilies[settings.fontFamily] ?? fontFamilies.system)
}

watch(
  settings,
  () => {
    localStorage.setItem(settingsStorageKey, JSON.stringify(settings))
    applySettings()
  },
  { deep: true, immediate: true }
)

// --- Logika gestów Swipe ---
const touchStartX = ref(0)
const touchStartY = ref(0)
const touchEndX = ref(0)
const touchEndY = ref(0)

const handleTouchStart = (e) => {
  touchStartX.value = e.changedTouches[0].screenX
  touchStartY.value = e.changedTouches[0].screenY
}

const handleTouchEnd = (e) => {
  touchEndX.value = e.changedTouches[0].screenX
  touchEndY.value = e.changedTouches[0].screenY
  checkSwipe()
}

const checkSwipe = () => {
  const xDiff = touchStartX.value - touchEndX.value
  const yDiff = touchStartY.value - touchEndY.value
  const threshold = 50

  if (Math.abs(yDiff) > Math.abs(xDiff)) return

  const currentIndex = tabs.findIndex((tab) => tab.id === activeTab.value)

  if (xDiff > threshold) {
    if (currentIndex < tabs.length - 1) activeTab.value = tabs[currentIndex + 1].id
  } else if (xDiff < -threshold) {
    if (currentIndex > 0) activeTab.value = tabs[currentIndex - 1].id
  }
}
</script>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: var(--app-font-family);
  font-size: var(--app-font-size);
  background: var(--app-bg);
  color: var(--app-text);
}

.app {
  width: 100%;
  height: 100dvh;
  display: flex;
  flex-direction: column;
  background: var(--app-surface);
  color: var(--app-text);
  font-family: var(--app-font-family);
  font-size: var(--app-font-size);
}

.app :where(h1, h2, h3, h4, h5, h6, p, span, small, strong, label, button, input, textarea, select, code, li, a, option) {
  font-family: inherit !important;
  font-size: inherit !important;
}

.main-title {
  padding: 20px;
  background: linear-gradient(135deg, var(--app-header-start) 0%, var(--app-header-end) 100%);
  color: white;
  margin: 0;
}

.tabs {
  display: flex;
  border-bottom: 2px solid var(--app-border);
  background: var(--app-surface-alt);
}

.tabs button {
  flex: 1;
  padding: 15px;
  background: none;
  border: none;
  font-size: 16px;
  cursor: pointer;
  transition: all 0.3s;
  font-weight: 500;
  color: var(--app-muted);
}

.tabs button:hover {
  background: var(--app-surface-alt);
}

.tabs button.active {
  background: var(--app-surface);
  color: var(--app-accent);
  border-bottom: 3px solid var(--app-accent);
}

.tab-content {
  padding: 20px;
  flex: 1;
  overflow-y: auto;
}

.app :where(h1, h2, h3, h4, h5, h6, p, span, small, strong, label, button, input, textarea, select, code, li, a) {
  font-family: inherit !important;
  font-size: inherit !important;
}

button {
  background: var(--app-accent);
  color: white;
  border: none;
  padding: 8px 16px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
  transition: all 0.3s;
}

button:hover {
  background: var(--app-accent-strong);
  transform: translateY(-1px);
}

button:disabled {
  background: #ccc;
  cursor: not-allowed;
  transform: none;
}

input,
textarea,
select {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid var(--app-border);
  border-radius: 6px;
  font-size: 14px;
  background: var(--app-surface);
  color: var(--app-text);
}

.card {
  background: var(--app-surface);
  border: 1px solid var(--app-border);
  border-radius: 8px;
  padding: 15px;
  margin-bottom: 15px;
}

.error {
  background: #fee;
  color: #c33;
  padding: 10px;
  border-radius: 6px;
  margin-bottom: 15px;
}

.success {
  background: #efe;
  color: #3c3;
  padding: 10px;
  border-radius: 6px;
  margin-bottom: 15px;
}

.grid {
  display: grid;
  gap: 15px;
}
</style>