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
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" :class="{ spinning: loading }">
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
          <circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/>
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

      <!-- List -->
      <div v-else key="list" class="coupons-grid">
        <div
          v-for="coupon in coupons"
          :key="coupon.coupon_id"
          class="coupon-card"
        >
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
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const props = defineProps({
  /** Passed from parent to trigger a refresh after a coupon is saved */
  refreshKey: {
    type: Number,
    default: 0,
  },
})

const coupons = ref([])
const loading = ref(false)
const error = ref(null)
const offset = ref(0)
const limit = ref(6)
const hasMore = ref(false)
const totalCount = ref(0)

const currentPage = computed(() => Math.floor(offset.value / limit.value) + 1)
const totalPages = computed(() => Math.ceil(totalCount.value / limit.value))

const loadCoupons = async (newOffset = 0) => {
  loading.value = true
  error.value = null
  try {
    const result = await invoke('load_coupons', {
      offset: Math.max(0, newOffset),
      limit: limit.value,
    })
    coupons.value = result.items
    offset.value = result.offset
    totalCount.value = result.total ?? 0
    hasMore.value = result.items.length === limit.value
  } catch (err) {
    error.value = typeof err === 'string' ? err : 'Failed to load coupons.'
    console.error(err)
  } finally {
    loading.value = false
  }
}

const refresh = () => loadCoupons(offset.value)

// Re-fetch from page 1 when refreshKey prop changes (parent triggers after save)
import { watch } from 'vue'
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

.toolbar-actions {
  display: flex;
  gap: 6px;
}

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

.state-view p { margin: 0; font-size: 15px; font-weight: 500; color: var(--text-secondary, #6b7280); }
.state-view small { font-size: 13px; }

.error-state { color: #dc2626; }
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

.coupon-code {
  font-family: 'Courier New', monospace;
  font-size: 15px;
  font-weight: 700;
  color: var(--text, #111827);
  padding: 12px 14px 6px;
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

/* ── Transitions ── */
.fade-enter-active, .fade-leave-active { transition: opacity 0.2s; }
.fade-enter-from, .fade-leave-to { opacity: 0; }

/* ── Responsive ── */
@media (max-width: 600px) {
  .coupons-grid { grid-template-columns: 1fr; }
}
</style>