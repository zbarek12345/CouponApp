<template>
  <div class="shop-view">

    <!-- Back -->
    <button class="btn-back" @click="$emit('back')">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <polyline points="15 18 9 12 15 6"/>
      </svg>
      {{ backLabel }}
    </button>

    <!-- Loading -->
    <div v-if="loading" class="sv-state">
      <div class="spinner"></div>
      <span>Loading shop…</span>
    </div>

    <div v-else-if="error" class="sv-error">{{ error }}</div>

    <template v-else-if="shop">

      <!-- ── Hero ── -->
      <div class="sv-hero">
        <div class="sv-logo">
          <img v-if="shop.logo_base64" :src="`data:image/png;base64,${shop.logo_base64}`" :alt="shop.shop_name" />
          <span v-else class="logo-initial">{{ shop.shop_name.charAt(0).toUpperCase() }}</span>
        </div>
        <div class="sv-hero-info">
          <h2 class="sv-name">{{ shop.shop_name }}</h2>
          <code class="sv-id-badge">{{ shop.shop_id }}</code>
        </div>
      </div>

      <!-- ── Tabs ── -->
      <div class="sv-tabs">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          class="sv-tab"
          :class="{ active: activeTab === tab.id }"
          @click="activeTab = tab.id"
        >
          {{ tab.icon }} {{ tab.label }}
          <span v-if="tab.id === 'coupons' && couponTotal !== null" class="tab-count">{{ couponTotal }}</span>
          <span v-if="tab.id === 'receipts' && receiptTotal !== null" class="tab-count">{{ receiptTotal }}</span>
        </button>
      </div>

      <!-- ── Coupons tab ── -->
      <div v-if="activeTab === 'coupons'" class="sv-section">
        <div v-if="couponsLoading" class="sv-state">
          <div class="spinner"></div>
        </div>
        <div v-else-if="couponsError" class="sv-error">{{ couponsError }}</div>
        <div v-else-if="coupons.length === 0" class="sv-empty">
          <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1">
            <path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z"/>
            <line x1="7" y1="7" x2="7.01" y2="7"/>
          </svg>
          <p>No coupons for this shop yet.</p>
        </div>
        <div v-else class="item-list">
          <button
            v-for="coupon in coupons"
            :key="coupon.coupon_id"
            class="item-row"
            @click="$emit('go-to-coupon', coupon.coupon_id)"
          >
            <div class="item-icon coupon-icon">
              <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">
                <path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z"/>
                <line x1="7" y1="7" x2="7.01" y2="7"/>
              </svg>
            </div>
            <div class="item-info">
              <strong>{{ coupon.description || 'Untitled Coupon' }}</strong>
              <small>{{ coupon.code_value }}</small>
            </div>
            <span class="item-badge">{{ coupon.code_type }}</span>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="item-chevron">
              <polyline points="9 18 15 12 9 6"/>
            </svg>
          </button>
        </div>

        <!-- Coupon pagination -->
        <div v-if="couponTotal > couponLimit" class="sv-pagination">
          <button class="pg-btn" @click="loadCoupons(couponOffset - couponLimit)" :disabled="couponOffset === 0">
            ← Prev
          </button>
          <span class="pg-info">{{ couponOffset + 1 }}–{{ Math.min(couponOffset + couponLimit, couponTotal) }} of {{ couponTotal }}</span>
          <button class="pg-btn" @click="loadCoupons(couponOffset + couponLimit)" :disabled="couponOffset + couponLimit >= couponTotal">
            Next →
          </button>
        </div>
      </div>

      <!-- ── Receipts tab ── -->
      <div v-if="activeTab === 'receipts'" class="sv-section">
        <div v-if="receiptsLoading" class="sv-state">
          <div class="spinner"></div>
        </div>
        <div v-else-if="receiptsError" class="sv-error">{{ receiptsError }}</div>
        <div v-else-if="receipts.length === 0" class="sv-empty">
          <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1">
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
            <polyline points="14 2 14 8 20 8"/>
          </svg>
          <p>No receipts for this shop yet.</p>
        </div>
        <div v-else class="item-list">
          <button
            v-for="receipt in receipts"
            :key="receipt.receipt_id"
            class="item-row"
            @click="$emit('go-to-receipt', receipt.receipt_id)"
          >
            <div class="item-icon receipt-icon">
              <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">
                <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
                <polyline points="14 2 14 8 20 8"/>
              </svg>
            </div>
            <div class="item-info">
              <strong>${{ receipt.total_value.toFixed(2) }}</strong>
              <small v-if="receipt.total_discount > 0" class="discount-note">
                – ${{ receipt.total_discount.toFixed(2) }} discount
              </small>
            </div>
            <code class="item-id-badge">{{ receipt.receipt_id.slice(0, 8) }}…</code>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="item-chevron">
              <polyline points="9 18 15 12 9 6"/>
            </svg>
          </button>
        </div>

        <!-- Receipt pagination -->
        <div v-if="receiptTotal > receiptLimit" class="sv-pagination">
          <button class="pg-btn" @click="loadReceipts(receiptOffset - receiptLimit)" :disabled="receiptOffset === 0">
            ← Prev
          </button>
          <span class="pg-info">{{ receiptOffset + 1 }}–{{ Math.min(receiptOffset + receiptLimit, receiptTotal) }} of {{ receiptTotal }}</span>
          <button class="pg-btn" @click="loadReceipts(receiptOffset + receiptLimit)" :disabled="receiptOffset + receiptLimit >= receiptTotal">
            Next →
          </button>
        </div>
      </div>

    </template>
  </div>
</template>

<script setup>
import { ref, watch, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const props = defineProps({
  shopId:    { type: String, required: true },
  backLabel: { type: String, default: 'All Shops' },
})

const emit = defineEmits(['back', 'go-to-coupon', 'go-to-receipt'])

// ── Shop ──
const shop    = ref(null)
const loading = ref(false)
const error   = ref(null)

// ── Tabs ──
const activeTab = ref('coupons')
const tabs = [
  { id: 'coupons',  icon: '🎫', label: 'Coupons'  },
  { id: 'receipts', icon: '🧾', label: 'Receipts' },
]

// ── Coupons ──
const coupons       = ref([])
const couponsLoading = ref(false)
const couponsError  = ref(null)
const couponOffset  = ref(0)
const couponLimit   = 10
const couponTotal   = ref(null)

// ── Receipts ──
const receipts       = ref([])
const receiptsLoading = ref(false)
const receiptsError  = ref(null)
const receiptOffset  = ref(0)
const receiptLimit   = 10
const receiptTotal   = ref(null)

// ── Loaders ──
const loadShop = async () => {
  loading.value = true
  error.value   = null
  try {
    shop.value = await invoke('load_shop', { shopId: props.shopId })
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

const loadCoupons = async (offset = 0) => {
  couponsLoading.value = true
  couponsError.value   = null
  try {
    const result = await invoke('load_coupons_for_shop', {
      shopId: props.shopId,
      offset,
      limit: couponLimit,
    })
    coupons.value      = result.items
    couponOffset.value = result.offset
    couponTotal.value  = result.total
  } catch (e) {
    couponsError.value = String(e)
  } finally {
    couponsLoading.value = false
  }
}

const loadReceipts = async (offset = 0) => {
  receiptsLoading.value = true
  receiptsError.value   = null
  try {
    const result = await invoke('load_receipts_for_shop', {
      shopId: props.shopId,
      offset,
      limit: receiptLimit,
    })
    receipts.value       = result.items
    receiptOffset.value  = result.offset
    receiptTotal.value   = result.total
  } catch (e) {
    receiptsError.value = String(e)
  } finally {
    receiptsLoading.value = false
  }
}

const init = async () => {
  await loadShop()
  await Promise.all([loadCoupons(), loadReceipts()])
}

watch(() => props.shopId, init)
onMounted(init)
</script>

<style scoped>
/* ── Layout ── */
.shop-view {
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

/* ── States ── */
.sv-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  padding: 40px 0;
  color: var(--app-muted);
}
.sv-error {
  padding: 14px 16px;
  background: rgba(255, 91, 91, 0.14);
  border: 1px solid rgba(255, 91, 91, 0.28);
  border-radius: 8px;
  color: #d64242;
  font-size: 14px;
  margin-bottom: 14px;
}
.sv-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  padding: 40px 0;
  color: var(--app-muted);
  font-size: 14px;
  text-align: center;
}
.sv-empty p { margin: 0; }

/* ── Hero ── */
.sv-hero {
  display: flex;
  align-items: center;
  gap: 18px;
  margin-bottom: 24px;
}

.sv-logo {
  width: 68px; height: 68px;
  border-radius: 16px;
  overflow: hidden;
  background: color-mix(in srgb, var(--app-accent) 12%, var(--app-surface));
  border: 1px solid var(--app-border);
  display: flex; align-items: center; justify-content: center;
  flex-shrink: 0;
}
.sv-logo img { width: 100%; height: 100%; object-fit: cover; }

.logo-initial {
  font-size: 28px;
  font-weight: 700;
  color: var(--app-accent);
}

.sv-hero-info {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.sv-name {
  margin: 0;
  font-size: 22px;
  font-weight: 800;
  color: var(--app-text);
  letter-spacing: 0;
}

.sv-id-badge {
  font-size: 11px;
  color: var(--app-muted);
  background: var(--app-surface-alt);
  padding: 2px 8px;
  border-radius: 4px;
  font-family: monospace;
  align-self: flex-start;
}

/* ── Tabs ── */
.sv-tabs {
  display: flex;
  gap: 2px;
  border-bottom: 2px solid var(--app-border);
  margin-bottom: 22px;
}

.sv-tab {
  display: flex;
  align-items: center;
  gap: 7px;
  padding: 10px 18px;
  background: none;
  border: none;
  border-bottom: 2px solid transparent;
  margin-bottom: -2px;
  font-size: 14px;
  font-weight: 600;
  color: var(--app-muted);
  cursor: pointer;
  transition: color 0.15s, border-color 0.15s;
  transform: none;
}
.sv-tab:hover { background: none; color: var(--app-text); transform: none; }
.sv-tab.active { color: var(--app-accent); border-bottom-color: var(--app-accent); background: none; }

.tab-count {
  background: color-mix(in srgb, var(--app-accent) 14%, transparent);
  color: var(--app-accent);
  font-size: 11px;
  font-weight: 700;
  padding: 1px 7px;
  border-radius: 10px;
}

/* ── Section ── */
.sv-section { }

/* ── Item list ── */
.item-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 16px;
}

.item-row {
  display: flex;
  align-items: center;
  gap: 13px;
  padding: 12px 14px;
  background: var(--app-surface);
  border: 1px solid var(--app-border);
  border-radius: 10px;
  cursor: pointer;
  text-align: left;
  width: 100%;
  color: var(--app-text);
  transition: border-color 0.15s, box-shadow 0.15s;
  transform: none;
}
.item-row:hover {
  border-color: var(--app-accent);
  box-shadow: 0 2px 8px rgba(102,126,234,0.1);
  background: var(--app-surface);
  transform: none;
}

.item-icon {
  width: 36px; height: 36px;
  border-radius: 8px;
  display: flex; align-items: center; justify-content: center;
  flex-shrink: 0;
}
.coupon-icon  { background: color-mix(in srgb, var(--app-accent) 14%, transparent); color: var(--app-accent); }
.receipt-icon { background: rgba(22, 163, 74, 0.13); color: #16a34a; }

.item-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
  overflow: hidden;
}
.item-info strong { font-size: 14px; font-weight: 600; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.item-info small  { font-size: 11px; color: var(--app-muted); font-family: monospace; }

.discount-note { color: #16a34a !important; font-family: inherit !important; font-size: 12px !important; }

.item-badge {
  font-size: 11px;
  font-weight: 700;
  background: color-mix(in srgb, var(--app-accent) 14%, transparent);
  color: var(--app-accent);
  padding: 3px 9px;
  border-radius: 20px;
  flex-shrink: 0;
}

.item-id-badge {
  font-family: monospace;
  font-size: 11px;
  color: var(--app-muted);
  flex-shrink: 0;
}

.item-chevron {
  color: var(--app-muted);
  flex-shrink: 0;
}
.item-row:hover .item-chevron { color: var(--app-accent); }

/* ── Pagination ── */
.sv-pagination {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 14px;
  padding: 8px 0 4px;
}

.pg-btn {
  padding: 7px 14px;
  border: 1px solid var(--app-border);
  border-radius: 8px;
  background: var(--app-surface);
  color: var(--app-muted);
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: border-color 0.15s, color 0.15s;
  transform: none;
}
.pg-btn:hover { border-color: var(--app-accent); color: var(--app-accent); transform: none; }
.pg-btn:disabled { opacity: 0.35; pointer-events: none; }

.pg-info { font-size: 13px; color: var(--app-muted); }

/* ── Spinner ── */
.spinner {
  width: 26px; height: 26px;
  border: 3px solid var(--app-border);
  border-top-color: var(--app-accent);
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}
@keyframes spin { to { transform: rotate(360deg); } }
</style>
