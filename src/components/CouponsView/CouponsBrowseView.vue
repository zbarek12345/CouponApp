<template>
  <div class="browse-view">
    <!-- Toolbar -->
    <div class="toolbar">
      <div class="page-info">
        <span class="page-label">Page {{ currentPage }}</span>
        <span v-if="totalCount" class="page-total">/ {{ totalPages }}</span>
        <span v-if="totalCount" class="total-count">{{ totalCount }} total</span>
      </div>

      <div class="toolbar-actions">
        <button class="icon-btn" title="First page" @click="loadCoupons(0)" :disabled="offset === 0 || loading">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <polyline points="19 20 9 12 19 4"/><line x1="5" y1="4" x2="5" y2="20"/>
          </svg>
        </button>
        <button class="icon-btn" title="Previous" @click="loadCoupons(offset - limit)" :disabled="offset === 0 || loading">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <polyline points="15 18 9 12 15 6"/>
          </svg>
        </button>
        <button class="icon-btn" title="Next" @click="loadCoupons(offset + limit)" :disabled="!hasMore || loading">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <polyline points="9 18 15 12 9 6"/>
          </svg>
        </button>
        <button class="icon-btn" title="Refresh" @click="refresh" :disabled="loading">
          <svg
            width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"
            :class="{ spinning: loading }"
          >
            <polyline points="23 4 23 10 17 10"/>
            <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/>
          </svg>
        </button>
      </div>
    </div>

    <!-- Body -->
    <transition name="fade" mode="out-in">
      <!-- Loading -->
      <div v-if="loading" key="loading" class="state-view">
        <div class="loader-ring" />
        <p>Loading coupons…</p>
      </div>

      <!-- Error -->
      <div v-else-if="error" key="error" class="state-view error-state">
        <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <circle cx="12" cy="12" r="10"/>
          <line x1="12" y1="8" x2="12" y2="12"/>
          <line x1="12" y1="16" x2="12.01" y2="16"/>
        </svg>
        <p>{{ error }}</p>
        <button class="retry-btn" @click="refresh">Retry</button>
      </div>

      <!-- Empty -->
      <div v-else-if="coupons.length === 0" key="empty" class="state-view empty-state">
        <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1">
          <rect x="2" y="6" width="20" height="12" rx="2"/>
          <line x1="8" y1="12" x2="16" y2="12"/>
          <circle cx="6" cy="12" r="1" fill="currentColor"/>
          <circle cx="18" cy="12" r="1" fill="currentColor"/>
        </svg>
        <p>No coupons yet</p>
        <small>Scan and assign coupons — they'll appear here.</small>
      </div>

      <!-- Grid -->
      <div v-else key="list" class="coupons-grid">
        <div
          v-for="coupon in coupons"
          :key="coupon.coupon_id"
          class="coupon-card"
        >
          <!-- Header -->
          <div class="coupon-header">
            <div class="shop-name">
              <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>
                <polyline points="9 22 9 12 15 12 15 22"/>
              </svg>
              {{ coupon.shop_name }}
            </div>
            <span class="type-pill">{{ coupon.code_type }}</span>
          </div>

          <!-- Rendered code image -->
          <div class="code-image-area">
            <!-- Not yet generated -->
            <template v-if="!codeImages[coupon.coupon_id]">
              <div class="code-placeholder">
                <button
                  class="render-btn"
                  @click="renderCode(coupon)"
                  :disabled="generatingIds.has(coupon.coupon_id)"
                >
                  <template v-if="generatingIds.has(coupon.coupon_id)">
                    <span class="btn-spinner small" />
                    Generating…
                  </template>
                  <template v-else>
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <rect x="3" y="3" width="18" height="18" rx="2"/>
                      <path d="M3 9h18M9 21V9"/>
                    </svg>
                    Render Code
                  </template>
                </button>
              </div>
            </template>

            <!-- Error -->
            <template v-else-if="codeImages[coupon.coupon_id].error">
              <div class="code-error">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <circle cx="12" cy="12" r="10"/>
                  <line x1="12" y1="8" x2="12" y2="12"/>
                  <line x1="12" y1="16" x2="12.01" y2="16"/>
                </svg>
                <span>{{ codeImages[coupon.coupon_id].error }}</span>
                <button class="retry-inline-btn" @click="renderCode(coupon, true)">Retry</button>
              </div>
            </template>

            <!-- Success: rendered image -->
            <template v-else>
              <div class="code-image-wrapper">
                <img
                  :src="`data:image/png;base64,${codeImages[coupon.coupon_id].b64}`"
                  :alt="`${coupon.code_type} code for ${coupon.code_value}`"
                  class="code-image"
                  draggable="false"
                />
                <button
                  class="regen-btn"
                  title="Regenerate"
                  @click="renderCode(coupon, true)"
                  :disabled="generatingIds.has(coupon.coupon_id)"
                >
                  <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                    <polyline points="23 4 23 10 17 10"/>
                    <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/>
                  </svg>
                </button>
              </div>
            </template>
          </div>

          <!-- Code value text -->
          <div class="coupon-code">{{ coupon.code_value }}</div>

          <p v-if="coupon.description" class="coupon-desc">{{ coupon.description }}</p>
          <p v-else class="coupon-desc muted">No description</p>

          <div class="coupon-footer">
            <span class="coupon-id">{{ coupon.coupon_id }}</span>
          </div>
        </div>
      </div>
    </transition>
  </div>
</template>

<script setup>
import { ref, computed, reactive, onMounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const props = defineProps({
  refreshKey: { type: Number, default: 0 },
})

// ── List state ────────────────────────────────────────────────────────────────
const coupons    = ref([])
const loading    = ref(false)
const error      = ref(null)
const offset     = ref(0)
const limit      = ref(6)
const hasMore    = ref(false)
const totalCount = ref(0)

const currentPage = computed(() => Math.floor(offset.value / limit.value) + 1)
const totalPages  = computed(() => Math.ceil(totalCount.value / limit.value))

// ── Code image cache ──────────────────────────────────────────────────────────
// keyed by coupon_id → { b64: string } | { error: string }
const codeImages   = reactive({})
// Set of coupon_ids currently being generated
const generatingIds = ref(new Set())

// ── Tauri calls ───────────────────────────────────────────────────────────────
const loadCoupons = async (newOffset = 0) => {
  loading.value = true
  error.value   = null
  try {
    const result = await invoke('load_coupons', {
      offset: Math.max(0, newOffset),
      limit:  limit.value,
    })
    coupons.value    = result.items
    offset.value     = result.offset
    totalCount.value = result.total ?? 0
    hasMore.value    = result.items.length === limit.value
  } catch (err) {
    error.value = typeof err === 'string' ? err : 'Failed to load coupons.'
    console.error(err)
  } finally {
    loading.value = false
  }
}

/**
 * Call generate_coupon_code_from_str for a single coupon.
 * @param {object} coupon  — must have coupon_id, code_value, code_type
 * @param {boolean} force  — bypass cache (retry / regen)
 */
const renderCode = async (coupon, force = false) => {
  const id = coupon.coupon_id
  if (generatingIds.value.has(id)) return
  if (!force && codeImages[id]?.b64) return   // already cached

  // Mark as generating (trigger reactive update)
  const next = new Set(generatingIds.value)
  next.add(id)
  generatingIds.value = next

  try {
    const b64 = await invoke('generate_coupon_code_from_str', {
      couponValue: coupon.code_value,
      couponType:  coupon.code_type,
    })
    codeImages[id] = { b64 }
  } catch (e) {
    codeImages[id] = { error: typeof e === 'string' ? e : 'Generation failed.' }
    console.error('generate_coupon_code_from_str:', e)
  } finally {
    const cleaned = new Set(generatingIds.value)
    cleaned.delete(id)
    generatingIds.value = cleaned
  }
}

const refresh = () => loadCoupons(offset.value)

watch(() => props.refreshKey, () => loadCoupons(0))
onMounted(() => loadCoupons(0))
</script>

<style scoped>
.browse-view {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

/* ── Toolbar ── */
.toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-bottom: 14px;
  border-bottom: 1px solid var(--border, #e5e7eb);
}

.page-info {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
}

.page-label { font-weight: 700; color: var(--text, #111827); }
.page-total { color: var(--text-muted, #9ca3af); }

.total-count {
  background: var(--bg, #f3f4f6);
  padding: 1px 8px;
  border-radius: 12px;
  font-size: 11px;
  font-weight: 600;
  color: var(--text-secondary, #374151);
}

.toolbar-actions { display: flex; gap: 6px; }

.icon-btn {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--border, #e5e7eb);
  border-radius: 7px;
  background: var(--surface, #fff);
  cursor: pointer;
  color: var(--text-secondary, #374151);
  transition: background 0.15s, color 0.15s;
}

.icon-btn:hover:not(:disabled) {
  background: var(--accent, #6366f1);
  color: #fff;
  border-color: var(--accent, #6366f1);
}

.icon-btn:disabled { opacity: 0.35; cursor: not-allowed; }

@keyframes spin { to { transform: rotate(360deg); } }
.spinning { animation: spin 0.8s linear infinite; }

/* ── State views ── */
.state-view {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
  gap: 10px;
  color: var(--text-muted, #9ca3af);
  text-align: center;
}

.state-view p     { margin: 0; font-size: 15px; font-weight: 500; color: var(--text-secondary, #6b7280); }
.state-view small { font-size: 13px; }

.error-state   { color: #dc2626; }
.error-state p { color: #dc2626; }

.loader-ring {
  width: 40px;
  height: 40px;
  border: 3px solid var(--border, #e5e7eb);
  border-top-color: var(--accent, #6366f1);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

.retry-btn {
  padding: 7px 18px;
  border: 1px solid #fca5a5;
  border-radius: 7px;
  background: #fef2f2;
  color: #dc2626;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: background 0.15s;
}
.retry-btn:hover { background: #fee2e2; }

/* ── Grid ── */
.coupons-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 14px;
}

.coupon-card {
  background: var(--surface, #fff);
  border: 1px solid var(--border, #e5e7eb);
  border-radius: 12px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  transition: box-shadow 0.2s, transform 0.15s;
}

.coupon-card:hover {
  box-shadow: 0 4px 16px rgba(0,0,0,0.08);
  transform: translateY(-2px);
}

/* ── Card header ── */
.coupon-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 14px;
  background: var(--bg, #f9fafb);
  border-bottom: 1px solid var(--border, #e5e7eb);
}

.shop-name {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  font-weight: 700;
  color: var(--text, #111827);
}

.type-pill {
  background: var(--accent, #6366f1);
  color: #fff;
  padding: 2px 8px;
  border-radius: 20px;
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

/* ── Code image area ── */
.code-image-area {
  background: var(--bg, #f9fafb);
  border-bottom: 1px solid var(--border, #e5e7eb);
  min-height: 80px;
  display: flex;
  align-items: center;
  justify-content: center;
}

/* Not-yet-generated placeholder */
.code-placeholder {
  padding: 14px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.render-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 7px 14px;
  background: var(--surface, #fff);
  border: 1px solid var(--border, #e5e7eb);
  border-radius: 8px;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary, #374151);
  cursor: pointer;
  transition: border-color 0.15s, color 0.15s, box-shadow 0.15s;
}

.render-btn:hover:not(:disabled) {
  border-color: var(--accent, #6366f1);
  color: var(--accent, #6366f1);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent, #6366f1) 10%, transparent);
}

.render-btn:disabled { opacity: 0.55; cursor: not-allowed; }

/* Error inline */
.code-error {
  display: flex;
  align-items: center;
  gap: 7px;
  padding: 12px 14px;
  color: #dc2626;
  font-size: 12px;
}

.code-error span { flex: 1; }

.retry-inline-btn {
  padding: 3px 10px;
  border: 1px solid #fca5a5;
  border-radius: 6px;
  background: #fef2f2;
  color: #dc2626;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  white-space: nowrap;
  transition: background 0.15s;
}
.retry-inline-btn:hover { background: #fee2e2; }

/* Rendered image */
.code-image-wrapper {
  width: 100%;
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 12px 14px;
  box-sizing: border-box;
}

.code-image {
  max-width: 100%;
  max-height: 120px;
  object-fit: contain;
  border-radius: 4px;
  /* Barcodes are typically white-background; add a subtle shadow so they
     stand out on both light and dark surfaces */
  box-shadow: 0 1px 4px rgba(0,0,0,0.12);
  image-rendering: pixelated; /* keep barcodes crisp when upscaled */
}

/* Small regen button overlaid top-right */
.regen-btn {
  position: absolute;
  top: 6px;
  right: 8px;
  width: 24px;
  height: 24px;
  border-radius: 6px;
  background: rgba(255,255,255,0.85);
  border: 1px solid var(--border, #e5e7eb);
  color: var(--text-muted, #9ca3af);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  opacity: 0;
  transition: opacity 0.15s, color 0.15s;
}

.code-image-wrapper:hover .regen-btn { opacity: 1; }
.regen-btn:hover { color: var(--accent, #6366f1); border-color: var(--accent, #6366f1); }
.regen-btn:disabled { cursor: not-allowed; opacity: 0.4; }

/* ── Code value text ── */
.coupon-code {
  font-family: 'Courier New', monospace;
  font-size: 13px;
  font-weight: 700;
  color: var(--text, #111827);
  padding: 10px 14px 4px;
  letter-spacing: 0.5px;
  word-break: break-all;
}

.coupon-desc {
  margin: 0;
  padding: 0 14px 12px;
  font-size: 13px;
  color: var(--text-secondary, #6b7280);
  flex: 1;
}

.coupon-desc.muted {
  color: var(--text-muted, #d1d5db);
  font-style: italic;
}

.coupon-footer {
  padding: 6px 14px;
  background: var(--bg, #fafafa);
  border-top: 1px solid var(--border, #f3f4f6);
}

.coupon-id {
  font-family: monospace;
  font-size: 10px;
  color: var(--text-muted, #d1d5db);
}

/* ── Spinner ── */
.btn-spinner {
  display: inline-block;
  width: 14px;
  height: 14px;
  border: 2px solid rgba(0,0,0,0.15);
  border-top-color: var(--accent, #6366f1);
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}

.btn-spinner.small {
  width: 12px;
  height: 12px;
}

/* ── Transitions ── */
.fade-enter-active, .fade-leave-active { transition: opacity 0.2s; }
.fade-enter-from, .fade-leave-to { opacity: 0; }

/* ── Responsive ── */
@media (max-width: 600px) {
  .coupons-grid { grid-template-columns: 1fr; }
}
</style>