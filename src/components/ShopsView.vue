<template>
  <div class="shops-view">

    <!-- ── BROWSER ── -->
    <template v-if="view === 'browser'">
      <div class="sv-header">
        <h2>Shops</h2>
        <div class="sv-header-actions">
          <button class="btn-ghost" @click="loadShops" :disabled="loading">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8"/><path d="M21 3v5h-5"/><path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16"/><path d="M8 16H3v5"/></svg>
            Refresh
          </button>
          <button @click="view = 'create'">+ New Shop</button>
        </div>
      </div>

      <div v-if="loading" class="sv-state">
        <div class="spinner"></div>
        <span>Loading…</span>
      </div>

      <div v-else-if="error" class="error">{{ error }}</div>

      <div v-else-if="shops.length === 0" class="sv-state sv-empty">
        <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1"><path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/></svg>
        <p>No shops yet.</p>
        <button @click="view = 'create'">Create your first shop</button>
      </div>

      <div v-else class="shop-list">
        <button
          v-for="shop in shops"
          :key="shop.shop_id"
          class="shop-row"
          @click="openShop(shop)"
        >
          <div class="shop-logo">
            <img v-if="shop.logo_base64" :src="`data:image/png;base64,${shop.logo_base64}`" :alt="shop.shop_name" />
            <span v-else class="logo-initial">{{ shop.shop_name.charAt(0).toUpperCase() }}</span>
          </div>
          <div class="shop-info">
            <strong>{{ shop.shop_name }}</strong>
            <small>{{ shop.shop_id }}</small>
          </div>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="chevron"><polyline points="9 18 15 12 9 6"/></svg>
        </button>
      </div>
    </template>

    <!-- ── CREATE ── -->
    <template v-else-if="view === 'create'">
      <button class="btn-back" @click="view = 'browser'">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="15 18 9 12 15 6"/></svg>
        Back
      </button>

      <div class="card">
        <h2>New Shop</h2>

        <div v-if="error" class="error">{{ error }}</div>

        <div class="field">
          <label>Shop Name *</label>
          <input v-model="form.name" placeholder="e.g. Green Market" :disabled="loading" @keydown.enter="createShop" />
        </div>

        <div class="field">
          <label>Logo</label>
          <div
            class="dropzone"
            :class="{ 'dropzone--filled': previewUrl, 'dropzone--over': isDragging }"
            @click="$refs.fileInput.click()"
            @dragover.prevent="isDragging = true"
            @dragleave="isDragging = false"
            @drop.prevent="onDrop"
          >
            <img v-if="previewUrl" :src="previewUrl" class="drop-preview" alt="Logo preview" />
            <div v-else class="drop-hint">
              <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="3" y="3" width="18" height="18" rx="2"/><circle cx="8.5" cy="8.5" r="1.5"/><polyline points="21 15 16 10 5 21"/></svg>
              <span>Click or drag image here</span>
            </div>
          </div>
          <input ref="fileInput" type="file" accept="image/*" style="display:none" @change="onFileChange" />
          <button v-if="previewUrl" class="btn-remove" @click.stop="clearLogo">Remove logo</button>
        </div>

        <div class="form-actions">
          <button class="btn-ghost" @click="view = 'browser'" :disabled="loading">Cancel</button>
          <button @click="createShop" :disabled="!form.name || loading">
            {{ loading ? 'Creating…' : 'Create Shop' }}
          </button>
        </div>
      </div>
    </template>

    <!-- ── SHOP DETAIL ── -->
    <template v-else-if="view === 'detail' && currentShop">
      <button class="btn-back" @click="view = 'browser'">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="15 18 9 12 15 6"/></svg>
        All Shops
      </button>

      <div class="detail-header">
        <div class="detail-logo">
          <img v-if="currentShop.logo_base64" :src="`data:image/png;base64,${currentShop.logo_base64}`" :alt="currentShop.shop_name" />
          <span v-else class="logo-initial logo-initial--lg">{{ currentShop.shop_name.charAt(0).toUpperCase() }}</span>
        </div>
        <div>
          <h2>{{ currentShop.shop_name }}</h2>
          <code class="shop-id-badge">{{ currentShop.shop_id }}</code>
        </div>
      </div>

      <!-- Detail tabs -->
      <div class="detail-tabs">
        <button
          v-for="dt in detailTabs"
          :key="dt.id"
          class="detail-tab"
          :class="{ active: activeDetailTab === dt.id }"
          @click="activeDetailTab = dt.id"
        >{{ dt.label }}</button>
      </div>

      <!-- Coupons -->
      <div v-if="activeDetailTab === 'coupons'" class="detail-section">
        <div class="section-toolbar">
          <span>Coupons</span>
          <button disabled>+ Add Coupon</button>
        </div>
        <div class="placeholder-list">
          <div v-for="n in 3" :key="n" class="placeholder-row shimmer">
            <div class="ph ph--icon"></div>
            <div class="ph-lines">
              <div class="ph ph--title"></div>
              <div class="ph ph--sub"></div>
            </div>
            <div class="ph ph--badge"></div>
          </div>
        </div>
        <p class="coming-soon">Coupon management coming soon.</p>
      </div>

      <!-- Receipts -->
      <div v-if="activeDetailTab === 'receipts'" class="detail-section">
        <div class="section-toolbar">
          <span>Receipts</span>
          <button disabled>+ Add Receipt</button>
        </div>
        <div class="placeholder-list">
          <div v-for="n in 4" :key="n" class="placeholder-row shimmer">
            <div class="ph ph--icon"></div>
            <div class="ph-lines">
              <div class="ph ph--title"></div>
              <div class="ph ph--sub"></div>
            </div>
            <div class="ph ph--amount"></div>
          </div>
        </div>
        <p class="coming-soon">Receipt tracking coming soon.</p>
      </div>
    </template>

  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// ── State ──────────────────────────────────────────────────────────────────
const view = ref('browser')           // 'browser' | 'create' | 'detail'
const shops = ref([])
const currentShop = ref(null)
const loading = ref(false)
const error = ref(null)

// Create form
const form = ref({ name: '', logo: null })
const previewUrl = ref(null)
const isDragging = ref(false)
const fileInput = ref(null)

// Detail tabs
const activeDetailTab = ref('coupons')
const detailTabs = [
  { id: 'coupons', label: '🎫 Coupons' },
  { id: 'receipts', label: '🧾 Receipts' },
]

// ── Shops CRUD ─────────────────────────────────────────────────────────────
const loadShops = async () => {
  loading.value = true
  error.value = null
  try {
    shops.value = await invoke('load_shops')
  } catch (err) {
    error.value = String(err)
  } finally {
    loading.value = false
  }
}

const createShop = async () => {
  if (!form.value.name) return
  loading.value = true
  error.value = null
  try {
    const shop = await invoke('create_shop', {
      request: { name: form.value.name, logo: form.value.logo ?? null },
    })
    await loadShops()
    openShop(shops.value.find(s => s.shop_id === shop.shop_id) ?? shop)
    resetForm()
  } catch (err) {
    error.value = String(err)
  } finally {
    loading.value = false
  }
}

const openShop = (shop) => {
  currentShop.value = shop
  activeDetailTab.value = 'coupons'
  view.value = 'detail'
}

// ── Logo helpers ───────────────────────────────────────────────────────────
const processFile = (file) => {
  if (!file?.type.startsWith('image/')) return
  const reader = new FileReader()
  reader.onload = (e) => {
    previewUrl.value = e.target.result
    form.value.logo = e.target.result.split(',')[1]   // raw base64 only
  }
  reader.readAsDataURL(file)
}

const onFileChange = (e) => processFile(e.target.files[0])
const onDrop = (e) => { isDragging.value = false; processFile(e.dataTransfer.files[0]) }
const clearLogo = () => {
  previewUrl.value = null
  form.value.logo = null
  if (fileInput.value) fileInput.value.value = ''
}
const resetForm = () => { form.value = { name: '', logo: null }; clearLogo() }

onMounted(loadShops)
</script>

<style scoped>
/* ── Layout ────────────────────────────────────────────────────────────── */
.shops-view { }

.sv-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 18px;
}
.sv-header h2 { font-size: 20px; font-weight: 700; margin: 0; }
.sv-header-actions { display: flex; gap: 8px; }

/* ── States ────────────────────────────────────────────────────────────── */
.sv-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  padding: 60px 0;
  color: #999;
  text-align: center;
}
.sv-empty p { margin: 0; }

.spinner {
  width: 28px; height: 28px;
  border: 3px solid #eee;
  border-top-color: #667eea;
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}
@keyframes spin { to { transform: rotate(360deg); } }

/* ── Shop list ─────────────────────────────────────────────────────────── */
.shop-list { display: flex; flex-direction: column; gap: 8px; }

.shop-row {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 12px 14px;
  background: #fff;
  border: 1px solid #e8e8e8;
  border-radius: 10px;
  cursor: pointer;
  text-align: left;
  width: 100%;
  color: inherit;
  transition: border-color 0.15s, box-shadow 0.15s;
}
.shop-row:hover {
  border-color: #667eea;
  box-shadow: 0 2px 8px rgba(102,126,234,0.12);
  transform: none;            /* override global button hover */
  background: #fff;
}

.shop-logo {
  width: 44px; height: 44px;
  border-radius: 8px;
  overflow: hidden;
  background: #f0f0ff;
  display: flex; align-items: center; justify-content: center;
  flex-shrink: 0;
}
.shop-logo img { width: 100%; height: 100%; object-fit: cover; }

.logo-initial {
  font-size: 18px; font-weight: 700; color: #667eea;
}
.logo-initial--lg { font-size: 28px; }

.shop-info { flex: 1; display: flex; flex-direction: column; gap: 2px; }
.shop-info strong { font-size: 15px; }
.shop-info small { font-size: 11px; color: #bbb; font-family: monospace; }

.chevron { color: #ccc; flex-shrink: 0; }
.shop-row:hover .chevron { color: #667eea; }

/* ── Back button ───────────────────────────────────────────────────────── */
.btn-back {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  background: none;
  border: none;
  color: #666;
  font-size: 14px;
  cursor: pointer;
  padding: 0 0 18px;
  font-weight: 500;
  transform: none;
}
.btn-back:hover { color: #111; background: none; transform: none; }

/* ── Ghost / remove buttons ────────────────────────────────────────────── */
.btn-ghost {
  background: transparent;
  color: #555;
  border: 1px solid #ddd;
  display: inline-flex;
  align-items: center;
  gap: 5px;
}
.btn-ghost:hover { background: #f5f5f5; transform: none; }
.btn-ghost:disabled { opacity: 0.4; }

.btn-remove {
  background: none;
  color: #e53e3e;
  border: none;
  padding: 4px 0;
  font-size: 13px;
  cursor: pointer;
  transform: none;
}
.btn-remove:hover { background: none; text-decoration: underline; transform: none; }

/* ── Create form ───────────────────────────────────────────────────────── */
.field { display: flex; flex-direction: column; gap: 6px; margin-bottom: 18px; }
label { font-size: 13px; font-weight: 600; color: #444; }

.dropzone {
  border: 2px dashed #ddd;
  border-radius: 10px;
  padding: 28px;
  text-align: center;
  cursor: pointer;
  transition: border-color 0.15s, background 0.15s;
  min-height: 110px;
  display: flex; align-items: center; justify-content: center;
}
.dropzone:hover, .dropzone--over { border-color: #667eea; background: rgba(102,126,234,0.03); }
.dropzone--filled { padding: 10px; border-style: solid; border-color: #c7c7ff; }

.drop-hint { display: flex; flex-direction: column; align-items: center; gap: 8px; color: #aaa; }
.drop-hint span { font-size: 13px; color: #777; }

.drop-preview { max-height: 100px; max-width: 100%; border-radius: 6px; object-fit: contain; }

.form-actions { display: flex; justify-content: flex-end; gap: 8px; margin-top: 4px; }

/* ── Detail header ─────────────────────────────────────────────────────── */
.detail-header {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-bottom: 24px;
}
.detail-logo {
  width: 64px; height: 64px;
  border-radius: 14px;
  overflow: hidden;
  background: #f0f0ff;
  border: 1px solid #e0e0ff;
  display: flex; align-items: center; justify-content: center;
  flex-shrink: 0;
}
.detail-logo img { width: 100%; height: 100%; object-fit: cover; }
.detail-header h2 { font-size: 22px; font-weight: 700; margin: 0 0 5px; }
.shop-id-badge {
  font-size: 11px; color: #bbb; background: #f5f5f5;
  padding: 2px 8px; border-radius: 4px; font-family: monospace;
}

/* ── Detail tabs ───────────────────────────────────────────────────────── */
.detail-tabs {
  display: flex;
  gap: 2px;
  border-bottom: 2px solid #e8e8e8;
  margin-bottom: 22px;
}
.detail-tab {
  padding: 9px 18px;
  background: none;
  border: none;
  border-bottom: 2px solid transparent;
  margin-bottom: -2px;
  font-size: 14px;
  font-weight: 500;
  color: #888;
  cursor: pointer;
  transition: color 0.15s, border-color 0.15s;
  transform: none;
}
.detail-tab:hover { background: none; color: #333; transform: none; }
.detail-tab.active { color: #667eea; border-bottom-color: #667eea; background: none; }

/* ── Detail sections ───────────────────────────────────────────────────── */
.detail-section { }
.section-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 14px;
  font-weight: 600;
  font-size: 15px;
}

/* ── Shimmer placeholders ──────────────────────────────────────────────── */
.placeholder-list { display: flex; flex-direction: column; gap: 8px; margin-bottom: 16px; }

@keyframes shimmer {
  0%   { background-position: -500px 0; }
  100% { background-position:  500px 0; }
}

.placeholder-row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px 16px;
  border-radius: 8px;
  border: 1px solid #eee;
}

.shimmer {
  background: linear-gradient(90deg, #f5f5f5 25%, #ebebeb 50%, #f5f5f5 75%);
  background-size: 500px 100%;
  animation: shimmer 1.3s infinite linear;
}

.ph { background: rgba(0,0,0,0.07); border-radius: 4px; }
.ph--icon  { width: 36px; height: 36px; border-radius: 8px; flex-shrink: 0; }
.ph-lines  { flex: 1; display: flex; flex-direction: column; gap: 7px; }
.ph--title { height: 13px; width: 140px; }
.ph--sub   { height: 10px; width: 90px; }
.ph--badge { width: 52px; height: 22px; border-radius: 11px; flex-shrink: 0; }
.ph--amount{ width: 58px; height: 14px; flex-shrink: 0; }

.coming-soon {
  text-align: center;
  color: #bbb;
  font-size: 13px;
  padding: 16px;
  border: 2px dashed #eee;
  border-radius: 8px;
}
</style>