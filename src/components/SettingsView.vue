<template>
  <div class="settings-view card">
    <div class="settings-hero">
      <div>
        <p class="eyebrow">Appearance</p>
        <h2>Settings</h2>
        <p>Change the appearance of the app</p>
      </div>
      <div class="theme-preview" :style="themePreviewStyle" />
    </div>

    <section class="settings-section">
      <div class="section-head">
        <h3>Mode</h3>
        <span>{{ modeLabel }}</span>
      </div>
      <div class="mode-switch">
        <button
          v-for="option in modeOptions"
          :key="option.id"
          class="mode-card"
          :class="{ active: settings.mode === option.id }"
          @click="setMode(option.id)"
        >
          <strong>{{ option.label }}</strong>
          <small>{{ option.description }}</small>
        </button>
      </div>
    </section>

    <section class="settings-section">
      <div class="section-head">
        <h3>Accent Theme</h3>
        <span>{{ themeLabel }}</span>
      </div>
      <div class="theme-grid">
        <button
          v-for="option in themeOptions"
          :key="option.id"
          class="theme-card"
          :class="{ active: settings.theme === option.id }"
          @click="setTheme(option.id)"
        >
          <span class="theme-swatch" :style="option.previewStyle" />
          <strong>{{ option.label }}</strong>
          <small>{{ option.description }}</small>
        </button>
      </div>
    </section>

    <section class="settings-section">
      <div class="section-head">
        <h3>Font Family</h3>
        <span>{{ fontLabel }}</span>
      </div>
      <div class="control-row">
        <select :value="settings.fontFamily" @change="setFontFamily($event.target.value)">
          <option v-for="option in fontOptions" :key="option.id" :value="option.id">
            {{ option.label }}
          </option>
        </select>
      </div>
    </section>

    <section class="settings-section">
      <div class="section-head">
        <h3>Font Size</h3>
        <span>{{ settings.fontSize }} px</span>
      </div>
      <div class="control-row control-row--slider">
        <input
          type="range"
          min="14"
          max="22"
          step="1"
          :value="settings.fontSize"
          @input="setFontSize(Number($event.target.value))"
        />
      </div>
    </section>

    <div class="settings-note">
    <center>
      Settings are imidiately applied.
    </center> 
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue'

const props = defineProps({
  settings: {
    type: Object,
    required: true,
  },
})

const emit = defineEmits(['update-settings'])

const modeOptions = [
  { id: 'light', label: 'Light mode'},
  { id: 'dark', label: 'Dark mode'},
]

const themeOptions = [
  {
    id: 'ocean',
    label: 'Ocean',
    description: 'Blue and violet accents',
    previewStyle: { background: 'linear-gradient(135deg, #4f7cff 0%, #7f5af0 100%)' },
  },
  {
    id: 'ember',
    label: 'Ember',
    description: 'Warm orange-red palette',
    previewStyle: { background: 'linear-gradient(135deg, #f16b49 0%, #ff9b71 100%)' },
  },
  {
    id: 'forest',
    label: 'Forest',
    description: 'Fresh green tones',
    previewStyle: { background: 'linear-gradient(135deg, #2f9e67 0%, #60b67e 100%)' },
  },
  {
    id: 'graphite',
    label: 'Graphite',
    description: 'Neutral gray with contrast',
    previewStyle: { background: 'linear-gradient(135deg, #4b5563 0%, #6b7280 100%)' },
  },
]

const fontOptions = [
  { id: 'system', label: 'System Sans' },
  { id: 'rounded', label: 'Rounded Sans' },
  { id: 'serif', label: 'Serif' },
  { id: 'mono', label: 'Monospace' },
]

const modeLabel = computed(() => modeOptions.find((option) => option.id === props.settings.mode)?.label ?? 'Light')
const themeLabel = computed(() => themeOptions.find((option) => option.id === props.settings.theme)?.label ?? 'Ocean')
const fontLabel = computed(() => fontOptions.find((option) => option.id === props.settings.fontFamily)?.label ?? 'System Sans')
const themePreviewStyle = computed(() => {
  const option = themeOptions.find((item) => item.id === props.settings.theme) ?? themeOptions[0]
  return option.previewStyle
})

const setMode = (mode) => {
  emit('update-settings', { ...props.settings, mode })
}

const setTheme = (theme) => {
  emit('update-settings', { ...props.settings, theme })
}

const setFontFamily = (fontFamily) => {
  emit('update-settings', { ...props.settings, fontFamily })
}

const setFontSize = (fontSize) => {
  emit('update-settings', { ...props.settings, fontSize })
}
</script>

<style scoped>
.settings-view {
  display: flex;
  flex-direction: column;
  gap: 18px;
}

.settings-hero {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 18px;
}

.eyebrow {
  text-transform: uppercase;
  letter-spacing: 0.12em;
  font-size: 11px;
  color: var(--app-muted);
  margin-bottom: 6px;
}

.settings-hero h2 {
  margin: 0;
  font-size: 26px;
}

.settings-hero p {
  margin-top: 8px;
  color: var(--app-muted);
  max-width: 44ch;
}

.theme-preview {
  width: 72px;
  height: 72px;
  border-radius: 18px;
  flex-shrink: 0;
  box-shadow: 0 12px 30px rgba(0, 0, 0, 0.12);
  background: linear-gradient(135deg, var(--app-header-start) 0%, var(--app-header-end) 100%);
}

.settings-section {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding-top: 4px;
}

.section-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
}

.section-head h3 {
  font-size: 16px;
  margin: 0;
}

.section-head span {
  color: var(--app-muted);
  font-size: 13px;
}

.theme-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
  gap: 12px;
}

.mode-switch {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
}

.mode-card {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 6px;
  text-align: left;
  padding: 14px;
  border: 1px solid var(--app-border);
  background: var(--app-surface);
  color: var(--app-text);
  transition: transform 0.15s, border-color 0.15s, box-shadow 0.15s;
}

.mode-card:hover {
  transform: translateY(-1px);
  border-color: var(--app-accent);
  box-shadow: 0 10px 24px rgba(0, 0, 0, 0.08);
}

.mode-card.active {
  border-color: var(--app-accent);
  box-shadow: 0 0 0 2px rgba(79, 124, 255, 0.18);
}

.mode-card small {
  color: var(--app-muted);
  line-height: 1.4;
}

.theme-card {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 8px;
  text-align: left;
  padding: 14px;
  border: 1px solid var(--app-border);
  background: var(--app-surface);
  color: var(--app-text);
  transition: transform 0.15s, border-color 0.15s, box-shadow 0.15s;
}

.theme-card:hover {
  transform: translateY(-1px);
  border-color: var(--app-accent);
  box-shadow: 0 10px 24px rgba(0, 0, 0, 0.08);
}

.theme-card.active {
  border-color: var(--app-accent);
  box-shadow: 0 0 0 2px rgba(79, 124, 255, 0.18);
}

.theme-swatch {
  width: 100%;
  height: 44px;
  border-radius: 12px;
}

.theme-card strong {
  font-size: 15px;
}

.theme-card small {
  color: var(--app-muted);
  line-height: 1.4;
}

.control-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.control-row--slider input {
  width: 100%;
}

.settings-note {
  color: var(--app-muted);
  font-size: 13px;
  padding-top: 4px;
}

select,
input[type='range'] {
  font: inherit;
}

@media (max-width: 700px) {
  .settings-hero {
    flex-direction: column;
  }

  .theme-preview {
    width: 100%;
    height: 14px;
    border-radius: 999px;
  }
}
</style>
