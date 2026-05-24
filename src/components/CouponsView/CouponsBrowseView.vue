<template>
  <div class="browse-view">
    <div class="toolbar">
      <div class="page-info">
        <span class="page-label">Page {{ currentPage }}</span>
        <span v-if="totalCount" class="page-total">/ {{ totalPages }}</span>
        <span v-if="totalCount" class="total-count">{{ totalCount }} total</span>
      </div>
      <div class="toolbar-actions">
        <button class="icon-btn" title="Previous" @click="loadCoupons(offset - limit)" :disabled="offset === 0 || loading">‹</button>
        <button class="icon-btn" title="Next" @click="loadCoupons(offset + limit)" :disabled="!hasMore || loading">›</button>
        <button class="icon-btn" title="Refresh" @click="refresh" :disabled="loading">↻</button>
      </div>
    </div>

    <div v-if="loading" class="state-view">
      <div class="loader-ring"></div>
      <p>Loading coupons...</p>
    </div>

    <div v-else-if="error" class="state-view error-state">
      <p>{{ error }}</p>
      <button class="retry-btn" @click="refresh">Retry</button>
    </div>

    <div v-else-if="coupons.length === 0" class="state-view empty-state">
      <p>No coupons yet</p>
      <small>Scan and assign coupons, then they will appear here.</small>
    </div>

    <div v-else class="coupons-grid">
      <button
        v-for="coupon in coupons"
        :key="coupon.coupon_id"
        class="coupon-card"
        type="button"
        @click="$emit('open-coupon', coupon.coupon_id)"
      >
        <div class="coupon-shop-art">
          <img
            v-if="shopLogo(coupon.shop_id)"
            :src="`data:image/png;base64,${shopLogo(coupon.shop_id)}`"
            :alt="coupon.shop_name"
          />
          <span v-else>{{ coupon.shop_name.charAt(0).toUpperCase() }}</span>
        </div>

        <div class="coupon-content">
          <div class="coupon-header">
            <span class="type-pill">{{ coupon.code_type }}</span>
            <span class="open-note">Open code</span>
          </div>

          <h3>{{ coupon.description || 'Untitled Coupon' }}</h3>
          <button class="shop-link" type="button" @click.stop="$emit('go-to-shop', coupon.shop_id)">
            {{ coupon.shop_name }}
          </button>
          <p class="coupon-code">{{ coupon.code_value }}</p>
        </div>
      </button>
    </div>
  </div>
</template>

<script setup>
import { computed, onMounted, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const props = defineProps({
  refreshKey: { type: Number, default: 0 },
})

defineEmits(['open-coupon', 'go-to-shop'])

const coupons = ref([])
const shops = ref([])
const loading = ref(false)
const error = ref(null)
const offset = ref(0)
const limit = ref(8)
const hasMore = ref(false)
const totalCount = ref(0)

const currentPage = computed(() => Math.floor(offset.value / limit.value) + 1)
const totalPages = computed(() => Math.max(1, Math.ceil(totalCount.value / limit.value)))

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
    hasMore.value = result.offset + result.items.length < result.total
  } catch (err) {
    error.value = typeof err === 'string' ? err : 'Failed to load coupons.'
  } finally {
    loading.value = false
  }
}

const loadShops = async () => {
  shops.value = await invoke('load_shops').catch(() => [])
}

const shopLogo = (shopId) => shops.value.find((shop) => shop.shop_id === shopId)?.logo_base64
const refresh = () => loadCoupons(offset.value)

watch(() => props.refreshKey, () => loadCoupons(0))

onMounted(async () => {
  await Promise.all([loadCoupons(0), loadShops()])
})
</script>

<style scoped>
.browse-view {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-bottom: 14px;
  border-bottom: 1px solid var(--app-border);
}

.page-info,
.toolbar-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.page-label {
  font-weight: 700;
}

.page-total {
  color: var(--app-muted);
}

.total-count {
  background: var(--app-surface-alt);
  color: var(--app-muted);
  padding: 2px 8px;
  border-radius: 12px;
  font-size: 11px;
  font-weight: 700;
}

.icon-btn {
  width: 32px;
  height: 32px;
  padding: 0;
  border: 1px solid var(--app-border);
  background: var(--app-surface);
  color: var(--app-text);
  border-radius: 7px;
  transform: none;
}

.state-view {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  padding: 60px 20px;
  color: var(--app-muted);
  text-align: center;
}

.error-state {
  color: #dc2626;
}

.loader-ring {
  width: 40px;
  height: 40px;
  border: 3px solid var(--app-border);
  border-top-color: var(--app-accent);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.coupons-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
  gap: 14px;
}

.coupon-card {
  display: grid;
  grid-template-rows: 136px 1fr;
  padding: 0;
  overflow: hidden;
  background: var(--app-surface);
  color: var(--app-text);
  border: 1px solid var(--app-border);
  border-radius: 8px;
  text-align: left;
  transform: none;
}

.coupon-card:hover {
  background: var(--app-surface);
  border-color: var(--app-accent);
  transform: none;
}

.coupon-shop-art {
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--app-surface-alt);
}

.coupon-shop-art img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.coupon-shop-art span {
  width: 76px;
  height: 76px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 8px;
  background: color-mix(in srgb, var(--app-accent) 14%, transparent);
  color: var(--app-accent);
  font-size: 32px;
  font-weight: 800;
}

.coupon-content {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 14px;
}

.coupon-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.type-pill {
  background: color-mix(in srgb, var(--app-accent) 14%, transparent);
  color: var(--app-accent);
  padding: 2px 8px;
  border-radius: 20px;
  font-size: 10px;
  font-weight: 800;
  text-transform: uppercase;
}

.open-note {
  color: var(--app-muted);
  font-size: 12px;
}

.coupon-content h3 {
  margin: 0;
  color: var(--app-text);
  font-size: 16px;
  line-height: 1.25;
}

.shop-link {
  width: fit-content;
  padding: 0;
  background: transparent;
  color: var(--app-accent);
  border: none;
  font-size: 13px;
  font-weight: 700;
  transform: none;
}

.shop-link:hover {
  background: transparent;
  text-decoration: underline;
  transform: none;
}

.coupon-code {
  margin: 0;
  color: var(--app-muted);
  font-family: monospace;
  font-size: 12px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.retry-btn {
  background: var(--app-surface);
  color: #dc2626;
  border: 1px solid #fca5a5;
}

@media (max-width: 600px) {
  .coupons-grid {
    grid-template-columns: 1fr;
  }
}
</style>
