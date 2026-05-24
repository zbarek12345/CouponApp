<template>
  <div class="coupon-view">

    <!-- Back button slot -->
    <button class="btn-back" @click="$emit('back')">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <polyline points="15 18 9 12 15 6"/>
      </svg>
      {{ backLabel }}
    </button>

    <!-- Loading -->
    <div v-if="loading" class="cv-state">
      <div class="spinner"></div>
      <span>Loading coupon…</span>
    </div>

    <div v-else-if="error" class="cv-error">{{ error }}</div>

    <template v-else-if="coupon">
      <!-- ── Hero ── -->
      <div class="cv-hero">
        <div class="cv-shop-logo" @click="$emit('go-to-shop', coupon.shop_id)">
          <img
            v-if="coupon.shop_logo_base64"
            :src="`data:image/png;base64,${coupon.shop_logo_base64}`"
            :alt="coupon.shop_name"
          />
          <span v-else>{{ coupon.shop_name.charAt(0).toUpperCase() }}</span>
        </div>

        <button class="cv-barcode-wrap" type="button" @click="expanded = true">
          <img
            v-if="barcodeImage"
            :src="`data:image/png;base64,${barcodeImage}`"
            class="cv-barcode"
            alt="Coupon barcode"
          />
          <div v-else-if="barcodeLoading" class="barcode-placeholder shimmer"></div>
          <div v-else class="barcode-error">
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <rect x="3" y="3" width="3" height="18"/><rect x="8" y="3" width="1" height="18"/>
              <rect x="11" y="3" width="2" height="18"/><rect x="15" y="3" width="1" height="18"/>
              <rect x="18" y="3" width="3" height="18"/>
            </svg>
            <span>Could not render barcode</span>
          </div>
        </button>

        <div class="cv-hero-info">
          <div class="cv-type-badge">{{ coupon.code_type }}</div>
          <h2 class="cv-title">{{ coupon.description || 'Untitled Coupon' }}</h2>
          <code class="cv-code-value">{{ coupon.code_value }}</code>
        </div>
      </div>

      <!-- ── Meta card ── -->
      <div class="cv-meta-card">
        <div class="cv-meta-row">
          <span class="cv-meta-label">Coupon ID</span>
          <code class="cv-meta-value mono">{{ coupon.coupon_id }}</code>
        </div>
        <div class="cv-meta-divider"></div>
        <div class="cv-meta-row">
          <span class="cv-meta-label">Barcode Format</span>
          <span class="cv-meta-value">{{ coupon.code_type }}</span>
        </div>
        <div class="cv-meta-divider"></div>
        <div class="cv-meta-row cv-meta-row--shop" @click="$emit('go-to-shop', coupon.shop_id)">
          <span class="cv-meta-label">Shop</span>
          <span class="cv-meta-value cv-shop-link">
            {{ coupon.shop_name }}
            <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
              <polyline points="9 18 15 12 9 6"/>
            </svg>
          </span>
        </div>
      </div>

      <!-- ── Actions ── -->
      <div class="cv-actions">
        <button class="cv-action-btn" @click="regenerateBarcode" :disabled="barcodeLoading">
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8"/>
            <path d="M21 3v5h-5"/>
            <path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16"/>
            <path d="M8 16H3v5"/>
          </svg>
          Regenerate
        </button>
        <button class="cv-action-btn" @click="copyCode">
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>
          </svg>
          {{ copied ? 'Copied!' : 'Copy Code' }}
        </button>
      </div>

      <div v-if="expanded" class="code-overlay" @click.self="expanded = false">
        <button class="overlay-close" type="button" @click="expanded = false">Close</button>
        <img
          v-if="barcodeImage"
          :src="`data:image/png;base64,${barcodeImage}`"
          class="expanded-code"
          alt="Expanded coupon barcode"
        />
        <p>{{ coupon.description || coupon.shop_name }}</p>
      </div>
    </template>

  </div>
</template>

<script setup>
import { ref, watch, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const props = defineProps({
  couponId: { type: String, required: true },
  backLabel: { type: String, default: 'Back' },
})

const emit = defineEmits(['back', 'go-to-shop'])

const coupon       = ref(null)
const loading      = ref(false)
const error        = ref(null)
const barcodeImage = ref(null)
const barcodeLoading = ref(false)
const copied       = ref(false)
const expanded     = ref(false)

const loadCoupon = async () => {
  loading.value = true
  error.value   = null
  coupon.value  = null
  barcodeImage.value = null
  expanded.value = false
  try {
    coupon.value = await invoke('load_coupon', { couponId: props.couponId })
    await loadBarcode()
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

const loadBarcode = async () => {
  if (!coupon.value) return
  barcodeLoading.value = true
  try {
    barcodeImage.value = await invoke('generate_coupon_code_from_str', {
      couponValue: coupon.value.code_value,
      couponType:  coupon.value.code_type,
    })
  } catch (e) {
    console.warn('Barcode render failed', e)
    barcodeImage.value = null
  } finally {
    barcodeLoading.value = false
  }
}

const regenerateBarcode = () => loadBarcode()

const copyCode = async () => {
  if (!coupon.value) return
  await navigator.clipboard.writeText(coupon.value.code_value).catch(() => {})
  copied.value = true
  setTimeout(() => { copied.value = false }, 2000)
}

watch(() => props.couponId, loadCoupon)
onMounted(loadCoupon)
</script>

<style scoped>
/* ── Layout ── */
.coupon-view {
  display: flex;
  flex-direction: column;
  gap: 0;
}

/* ── Back ── */
.btn-back {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  background: none;
  border: none;
  color: var(--app-muted);
  font-size: 14px;
  cursor: pointer;
  padding: 0 0 20px;
  font-weight: 500;
  transform: none;
}
.btn-back:hover { color: var(--app-text); background: none; transform: none; }

/* ── State ── */
.cv-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  padding: 60px 0;
  color: var(--app-muted);
}
.cv-error {
  padding: 14px 16px;
  background: rgba(255, 91, 91, 0.14);
  border: 1px solid rgba(255, 91, 91, 0.28);
  border-radius: 8px;
  color: #d64242;
  font-size: 14px;
}

/* ── Hero ── */
.cv-hero {
  background: var(--app-surface);
  border: 1px solid var(--app-border);
  border-radius: 14px;
  padding: 28px 24px 24px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 22px;
  margin-bottom: 14px;
}

.cv-shop-logo {
  width: 86px;
  height: 86px;
  border-radius: 8px;
  overflow: hidden;
  background: color-mix(in srgb, var(--app-accent) 14%, transparent);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
}

.cv-shop-logo img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.cv-shop-logo span {
  color: var(--app-accent);
  font-size: 34px;
  font-weight: 800;
}

.cv-barcode-wrap {
  width: 100%;
  max-width: 340px;
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 90px;
  padding: 12px;
  background: #fff;
  border: 1px solid var(--app-border);
  border-radius: 8px;
  transform: none;
}

.cv-barcode {
  max-width: 100%;
  max-height: 120px;
  object-fit: contain;
  image-rendering: pixelated;
  border-radius: 4px;
}

.barcode-placeholder {
  width: 260px;
  height: 80px;
  border-radius: 6px;
}

.barcode-error {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  color: var(--app-muted);
  font-size: 13px;
}

.cv-hero-info {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  text-align: center;
}

.cv-type-badge {
  display: inline-block;
  padding: 3px 10px;
  background: color-mix(in srgb, var(--app-accent) 14%, transparent);
  color: var(--app-accent);
  border-radius: 20px;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.05em;
  text-transform: uppercase;
}

.cv-title {
  margin: 0;
  font-size: 20px;
  font-weight: 700;
  color: var(--app-text);
  letter-spacing: 0;
}

.cv-code-value {
  font-family: 'Courier New', monospace;
  font-size: 13px;
  color: var(--app-accent);
  background: color-mix(in srgb, var(--app-accent) 12%, transparent);
  padding: 4px 12px;
  border-radius: 6px;
  word-break: break-all;
  text-align: center;
}

/* ── Meta card ── */
.cv-meta-card {
  background: var(--app-surface);
  border: 1px solid var(--app-border);
  border-radius: 12px;
  overflow: hidden;
  margin-bottom: 14px;
}

.cv-meta-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 13px 16px;
  gap: 12px;
}

.cv-meta-row--shop {
  cursor: pointer;
  transition: background 0.12s;
}
.cv-meta-row--shop:hover { background: var(--app-surface-alt); }

.cv-meta-divider {
  height: 1px;
  background: var(--app-border);
  margin: 0 16px;
}

.cv-meta-label {
  font-size: 13px;
  color: var(--app-muted);
  font-weight: 500;
  flex-shrink: 0;
}

.cv-meta-value {
  font-size: 13px;
  color: var(--app-text);
  font-weight: 500;
  text-align: right;
  word-break: break-all;
}
.cv-meta-value.mono {
  font-family: 'Courier New', monospace;
  font-size: 11px;
  color: var(--app-muted);
}

.cv-shop-link {
  display: flex;
  align-items: center;
  gap: 4px;
  color: var(--app-accent);
  font-weight: 600;
}

/* ── Actions ── */
.cv-actions {
  display: flex;
  gap: 10px;
}

.cv-action-btn {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 7px;
  padding: 11px 16px;
  border: 1px solid var(--app-border);
  border-radius: 10px;
  background: var(--app-surface);
  color: var(--app-text);
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: border-color 0.15s, background 0.15s, color 0.15s;
  transform: none;
}
.cv-action-btn:hover {
  border-color: var(--app-accent);
  color: var(--app-accent);
  background: var(--app-surface-alt);
  transform: none;
}
.cv-action-btn:disabled { opacity: 0.4; pointer-events: none; }

.code-overlay {
  position: fixed;
  inset: 0;
  z-index: 100;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 18px;
  padding: 24px;
  background: color-mix(in srgb, var(--app-surface) 96%, transparent);
}

.expanded-code {
  max-width: min(92vw, 900px);
  max-height: 72vh;
  object-fit: contain;
  image-rendering: pixelated;
}

.overlay-close {
  position: fixed;
  top: 18px;
  right: 18px;
}

.code-overlay p {
  margin: 0;
  color: var(--app-text);
  font-size: 18px;
  font-weight: 700;
  text-align: center;
}

/* ── Shimmer ── */
@keyframes shimmer {
  0%   { background-position: -500px 0; }
  100% { background-position:  500px 0; }
}
.shimmer {
  background: linear-gradient(90deg, var(--app-surface-alt) 25%, var(--app-border) 50%, var(--app-surface-alt) 75%);
  background-size: 500px 100%;
  animation: shimmer 1.3s infinite linear;
}

/* ── Spinner ── */
.spinner {
  width: 28px; height: 28px;
  border: 3px solid var(--app-border);
  border-top-color: var(--app-accent);
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}
@keyframes spin { to { transform: rotate(360deg); } }
</style>
