<template>
  <div class="create-view">
    <button class="back-btn" @click="$emit('navigate', 'browser')">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="15 18 9 12 15 6"/></svg>
      Back to Shops
    </button>

    <div class="form-card">
      <h1>New Shop</h1>
      <p class="subtitle">Fill in the details below to register a new shop.</p>

      <div v-if="error" class="alert-error">{{ error }}</div>

      <div class="field">
        <label for="shop-name">Shop Name <span class="required">*</span></label>
        <input
          id="shop-name"
          v-model="form.name"
          placeholder="e.g. Green Market"
          :disabled="loading"
          @keydown.enter="submit"
        />
      </div>

      <div class="field">
        <label>Logo</label>
        <div
          class="logo-dropzone"
          :class="{ 'has-image': previewUrl, 'drag-over': isDragging }"
          @click="triggerFileInput"
          @dragover.prevent="isDragging = true"
          @dragleave="isDragging = false"
          @drop.prevent="onDrop"
        >
          <img v-if="previewUrl" :src="previewUrl" class="logo-preview" alt="Logo preview" />
          <div v-else class="dropzone-placeholder">
            <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="3" y="3" width="18" height="18" rx="2"/><circle cx="8.5" cy="8.5" r="1.5"/><polyline points="21 15 16 10 5 21"/></svg>
            <span>Click or drag an image</span>
            <small>PNG, JPG up to 4 MB</small>
          </div>
        </div>
        <input
          ref="fileInput"
          type="file"
          accept="image/png,image/jpeg,image/webp"
          style="display:none"
          @change="onFileChange"
        />
        <button v-if="previewUrl" class="remove-logo" @click.stop="clearLogo">Remove logo</button>
      </div>

      <div class="form-actions">
        <button class="btn-ghost" @click="$emit('navigate', 'browser')" :disabled="loading">Cancel</button>
        <button class="btn-primary" @click="submit" :disabled="!form.name || loading">
          <span v-if="loading" class="btn-spinner"></span>
          {{ loading ? 'Creating…' : 'Create Shop' }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const emit = defineEmits(['navigate'])

const form = ref({ name: '', logo: null })
const previewUrl = ref(null)
const loading = ref(false)
const error = ref(null)
const isDragging = ref(false)
const fileInput = ref(null)

const triggerFileInput = () => fileInput.value?.click()

const processFile = (file) => {
  if (!file || !file.type.startsWith('image/')) return
  const reader = new FileReader()
  reader.onload = (e) => {
    const result = e.target.result          // data:image/...;base64,XXXX
    previewUrl.value = result
    // Strip the data URL prefix — backend expects raw base64
    form.value.logo = result.split(',')[1]
  }
  reader.readAsDataURL(file)
}

const onFileChange = (e) => processFile(e.target.files[0])
const onDrop = (e) => {
  isDragging.value = false
  processFile(e.dataTransfer.files[0])
}
const clearLogo = () => {
  previewUrl.value = null
  form.value.logo = null
  if (fileInput.value) fileInput.value.value = ''
}

const submit = async () => {
  if (!form.value.name) return
  loading.value = true
  error.value = null
  try {
    const shop = await invoke('create_shop', {
      request: {
        name: form.value.name,
        logo: form.value.logo ?? null,
      },
    })
    emit('navigate', 'shop', shop)
  } catch (err) {
    error.value = String(err)
    console.error(err)
  } finally {
    loading.value = false
  }
}
</script>

<style scoped>
.create-view {
  padding: 32px;
  max-width: 560px;
  margin: 0 auto;
}

.back-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  background: none;
  border: none;
  color: #666;
  font-size: 14px;
  cursor: pointer;
  padding: 0;
  margin-bottom: 28px;
  transition: color 0.15s;
}
.back-btn:hover { color: #111; }

.form-card {
  background: #fff;
  border: 1px solid #eee;
  border-radius: 16px;
  padding: 32px;
}

.form-card h1 {
  font-size: 24px;
  font-weight: 700;
  margin: 0 0 6px;
  letter-spacing: -0.5px;
}
.subtitle { color: #888; font-size: 14px; margin: 0 0 28px; }

.alert-error {
  background: #fff5f5;
  border: 1px solid #fed7d7;
  color: #c53030;
  padding: 10px 14px;
  border-radius: 8px;
  font-size: 13px;
  margin-bottom: 20px;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 22px;
}
label {
  font-size: 13px;
  font-weight: 600;
  color: #444;
}
.required { color: #e53e3e; }

input[type="text"], input:not([type]) {
  width: 100%;
  padding: 10px 14px;
  border: 1px solid #ddd;
  border-radius: 8px;
  font-size: 14px;
  outline: none;
  transition: border-color 0.15s, box-shadow 0.15s;
  box-sizing: border-box;
}
input:focus {
  border-color: #4f46e5;
  box-shadow: 0 0 0 3px rgba(79,70,229,0.1);
}
input:disabled { background: #f7f7f7; color: #aaa; }

/* Dropzone */
.logo-dropzone {
  border: 2px dashed #ddd;
  border-radius: 12px;
  padding: 32px;
  text-align: center;
  cursor: pointer;
  transition: border-color 0.15s, background 0.15s;
  min-height: 130px;
  display: flex;
  align-items: center;
  justify-content: center;
}
.logo-dropzone:hover, .logo-dropzone.drag-over {
  border-color: #4f46e5;
  background: rgba(79,70,229,0.03);
}
.logo-dropzone.has-image {
  padding: 12px;
  border-style: solid;
  border-color: #c7c7ff;
}

.dropzone-placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  color: #aaa;
}
.dropzone-placeholder span { font-size: 14px; color: #666; }
.dropzone-placeholder small { font-size: 12px; }

.logo-preview {
  max-height: 120px;
  max-width: 100%;
  border-radius: 8px;
  object-fit: contain;
}

.remove-logo {
  background: none;
  border: none;
  color: #e53e3e;
  font-size: 13px;
  cursor: pointer;
  padding: 0;
  align-self: flex-start;
}
.remove-logo:hover { text-decoration: underline; }

/* Actions */
.form-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  margin-top: 8px;
}

.btn-primary {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 10px 20px;
  background: #4f46e5;
  color: #fff;
  border: none;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.15s;
}
.btn-primary:hover:not(:disabled) { background: #4338ca; }
.btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }

.btn-ghost {
  padding: 10px 16px;
  background: transparent;
  color: #555;
  border: 1px solid #ddd;
  border-radius: 8px;
  font-size: 14px;
  cursor: pointer;
  transition: border-color 0.15s;
}
.btn-ghost:hover:not(:disabled) { border-color: #aaa; }
.btn-ghost:disabled { opacity: 0.4; cursor: not-allowed; }

.btn-spinner {
  width: 14px;
  height: 14px;
  border: 2px solid rgba(255,255,255,0.4);
  border-top-color: #fff;
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
  display: inline-block;
}
@keyframes spin { to { transform: rotate(360deg); } }
</style>