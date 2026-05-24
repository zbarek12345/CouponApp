<template>
  <div class="shops-view">
    <ShopDetail
      v-if="view === 'detail' && currentShopId"
      :shop-id="currentShopId"
      back-label="All Shops"
      @back="backToBrowser"
      @go-to-coupon="emit('go-to-coupon', $event)"
      @go-to-receipt="emit('go-to-receipt', $event)"
    />

    <template v-else-if="view === 'create'">
      <button class="btn-back" @click="backToBrowser">Back</button>
      <CreateShop @navigate="onCreateNavigate" />
    </template>

    <template v-else>
      <div class="sv-header">
        <h2>Shops</h2>
        <div class="sv-header-actions">
          <button class="btn-ghost icon-btn" @click="loadShops" :disabled="loading" title="Refresh">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8"/><path d="M21 3v5h-5"/><path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16"/><path d="M8 16H3v5"/></svg>
          </button>
          <button class="new-shop-btn" @click="view = 'create'">+ New</button>
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

      <template v-else>
        <section v-if="topShops.length" class="top-shops" aria-label="Top shops">
          <div class="section-head">
            <div>
              <h3>Top shops</h3>
              <span>Ranked by coupons and receipts</span>
            </div>
            <div v-if="topShops.length > 1" class="carousel-controls">
              <button class="btn-ghost icon-btn" type="button" @click="previousTopShop" title="Previous shop">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="15 18 9 12 15 6"/></svg>
              </button>
              <button class="btn-ghost icon-btn" type="button" @click="nextTopShop" title="Next shop">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="9 18 15 12 9 6"/></svg>
              </button>
            </div>
          </div>

          <div class="top-carousel" @touchstart.stop @touchmove.stop @touchend.stop>
            <button
              v-for="(shop, index) in topShops"
              :key="shop.shop_id"
              class="top-card"
              :class="{ active: index === activeTopIndex }"
              :style="{ transform: `translateX(${(index - activeTopIndex) * 100}%)` }"
              type="button"
              @click="openShop(shop.shop_id)"
            >
              <div class="top-rank">#{{ index + 1 }}</div>
              <div class="shop-logo top-logo">
                <img v-if="shop.logo_base64" :src="`data:image/png;base64,${shop.logo_base64}`" :alt="shop.shop_name" />
                <span v-else class="logo-initial">{{ shop.shop_name.charAt(0).toUpperCase() }}</span>
              </div>
              <div class="top-info">
                <strong>{{ shop.shop_name }}</strong>
                <span>{{ shop.activityTotal }} added</span>
              </div>
              <div class="top-stats">
                <span>{{ shop.couponTotal }} coupons</span>
                <span>{{ shop.receiptTotal }} receipts</span>
              </div>
            </button>
          </div>

          <div v-if="topShops.length > 1" class="carousel-dots" aria-label="Top shop slides">
            <button
              v-for="(shop, index) in topShops"
              :key="`${shop.shop_id}-dot`"
              class="dot"
              :class="{ active: index === activeTopIndex }"
              type="button"
              :title="`Show ${shop.shop_name}`"
              @click="activeTopIndex = index"
            />
          </div>
        </section>

        <div class="shop-search">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>
          <input v-model="searchQuery" type="search" placeholder="Search shops" />
        </div>

        <div v-if="filteredShops.length === 0" class="sv-state sv-empty">
          <p>No shops match your search.</p>
        </div>

        <div v-else class="shop-list">
          <button
            v-for="shop in filteredShops"
            :key="shop.shop_id"
            class="shop-row"
            @click="openShop(shop.shop_id)"
          >
            <div class="shop-logo">
              <img v-if="shop.logo_base64" :src="`data:image/png;base64,${shop.logo_base64}`" :alt="shop.shop_name" />
              <span v-else class="logo-initial">{{ shop.shop_name.charAt(0).toUpperCase() }}</span>
            </div>
            <div class="shop-info">
              <strong>{{ shop.shop_name }}</strong>
              <small v-if="shopStats[shop.shop_id]">
                {{ shopStats[shop.shop_id].couponTotal }} coupons &middot; {{ shopStats[shop.shop_id].receiptTotal }} receipts
              </small>
            </div>
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="chevron"><polyline points="9 18 15 12 9 6"/></svg>
          </button>
        </div>
      </template>
    </template>
  </div>
</template>

<script setup>
import { computed, onMounted, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import ShopDetail from './ShopsView/ShopDetail.vue'
import CreateShop from './ShopsView/CreateShop.vue'

const props = defineProps({
  selectedShopId: { type: String, default: null },
})

const emit = defineEmits(['go-to-coupon', 'go-to-receipt'])

const view = ref('browser')
const shops = ref([])
const shopStats = ref({})
const currentShopId = ref(null)
const loading = ref(false)
const error = ref(null)
const searchQuery = ref('')
const activeTopIndex = ref(0)

const shopsWithStats = computed(() => shops.value.map((shop) => {
  const stats = shopStats.value[shop.shop_id] ?? { couponTotal: 0, receiptTotal: 0 }

  return {
    ...shop,
    couponTotal: stats.couponTotal,
    receiptTotal: stats.receiptTotal,
    activityTotal: stats.couponTotal + stats.receiptTotal,
  }
}))

const topShops = computed(() => shopsWithStats.value
  .filter((shop) => shop.activityTotal > 0)
  .sort((a, b) => b.activityTotal - a.activityTotal || a.shop_name.localeCompare(b.shop_name))
  .slice(0, 5))

const filteredShops = computed(() => {
  const query = searchQuery.value.trim().toLowerCase()
  if (!query) return shopsWithStats.value

  return shopsWithStats.value.filter((shop) => shop.shop_name.toLowerCase().includes(query))
})

const loadShopStats = async (loadedShops) => {
  const entries = await Promise.all(loadedShops.map(async (shop) => {
    try {
      const [coupons, receipts] = await Promise.all([
        invoke('load_coupons_for_shop', { shopId: shop.shop_id, offset: 0, limit: 1 }),
        invoke('load_receipts_for_shop', { shopId: shop.shop_id, offset: 0, limit: 1 }),
      ])

      return [
        shop.shop_id,
        {
          couponTotal: Number(coupons.total ?? 0),
          receiptTotal: Number(receipts.total ?? 0),
        },
      ]
    } catch {
      return [shop.shop_id, { couponTotal: 0, receiptTotal: 0 }]
    }
  }))

  shopStats.value = Object.fromEntries(entries)
}

const loadShops = async () => {
  loading.value = true
  error.value = null
  try {
    const loadedShops = await invoke('load_shops')
    shops.value = loadedShops
    await loadShopStats(loadedShops)
    activeTopIndex.value = 0
  } catch (err) {
    error.value = String(err)
  } finally {
    loading.value = false
  }
}

const openShop = (shopId) => {
  currentShopId.value = shopId
  view.value = 'detail'
}

const backToBrowser = async () => {
  currentShopId.value = null
  view.value = 'browser'
  await loadShops()
}

const onCreateNavigate = (target, shop) => {
  if (target === 'shop' && shop?.shop_id) {
    currentShopId.value = shop.shop_id
    view.value = 'detail'
  } else {
    backToBrowser()
  }
}

const previousTopShop = () => {
  if (!topShops.value.length) return
  activeTopIndex.value = (activeTopIndex.value - 1 + topShops.value.length) % topShops.value.length
}

const nextTopShop = () => {
  if (!topShops.value.length) return
  activeTopIndex.value = (activeTopIndex.value + 1) % topShops.value.length
}

watch(
  () => props.selectedShopId,
  (shopId) => {
    if (shopId) openShop(shopId)
  },
  { immediate: true }
)

onMounted(loadShops)
</script>

<style scoped>
.sv-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 18px;
}

.sv-header h2 {
  margin: 0;
  font-size: 20px;
}

.sv-header-actions {
  display: flex;
  gap: 8px;
  align-items: center;
}

.icon-btn {
  padding: 8px !important;
  display: flex;
  align-items: center;
  justify-content: center;
}

.btn-ghost {
  background: transparent;
  color: var(--app-accent);
  border: 1px solid transparent;
}

.btn-ghost:hover {
  background: color-mix(in srgb, var(--app-accent) 12%, transparent);
}

.new-shop-btn {
  padding: 8px 12px;
}

.sv-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  padding: 60px 0;
  color: var(--app-muted);
  text-align: center;
}

.sv-empty p { margin: 0; }

.spinner {
  width: 28px;
  height: 28px;
  border: 3px solid var(--app-border);
  border-top-color: var(--app-accent);
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.top-shops {
  margin-bottom: 18px;
}

.section-head {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  align-items: center;
  margin-bottom: 10px;
}

.section-head h3 {
  margin: 0 0 2px;
  font-size: 15px;
  color: var(--app-text);
}

.section-head span {
  color: var(--app-muted);
  font-size: 12px !important;
}

.carousel-controls {
  display: flex;
  gap: 6px;
}

.top-carousel {
  position: relative;
  height: 190px;
  overflow: hidden;
  border-radius: 10px;
  border: 1px solid var(--app-border);
  background: var(--app-surface-alt);
}

.top-card {
  position: absolute;
  inset: 0;
  display: grid;
  grid-template-columns: auto 1fr;
  grid-template-rows: auto 1fr auto;
  gap: 10px 14px;
  align-items: center;
  width: 100%;
  padding: 18px;
  background: var(--app-surface);
  color: var(--app-text);
  border: 0;
  border-radius: 0;
  cursor: pointer;
  opacity: 0;
  text-align: left;
  transition: transform 0.25s ease, opacity 0.2s ease;
}

.top-card.active {
  opacity: 1;
  z-index: 1;
}

.top-card:hover {
  background: var(--app-surface);
}

.top-rank {
  grid-column: 1 / -1;
  justify-self: start;
  padding: 3px 9px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--app-accent) 14%, transparent);
  color: var(--app-accent);
  font-size: 12px;
  font-weight: 800;
}

.top-logo {
  width: 78px !important;
  height: 78px !important;
  border-radius: 14px !important;
}

.top-info {
  min-width: 0;
}

.top-info strong {
  display: block;
  font-size: 20px !important;
  line-height: 1.2;
  margin-bottom: 4px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.top-info span {
  color: var(--app-muted);
  font-size: 13px !important;
}

.top-stats {
  grid-column: 1 / -1;
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.top-stats span {
  padding: 5px 9px;
  border: 1px solid var(--app-border);
  border-radius: 999px;
  background: var(--app-surface-alt);
  color: var(--app-muted);
  font-size: 12px !important;
  font-weight: 700;
}

.carousel-dots {
  display: flex;
  justify-content: center;
  gap: 7px;
  margin-top: 10px;
}

.dot {
  width: 8px;
  height: 8px;
  padding: 0;
  border-radius: 999px;
  border: 0;
  background: var(--app-border);
}

.dot.active {
  width: 22px;
  background: var(--app-accent);
}

.shop-search {
  position: relative;
  display: flex;
  align-items: center;
  margin-bottom: 12px;
}

.shop-search svg {
  position: absolute;
  left: 12px;
  color: var(--app-muted);
  pointer-events: none;
}

.shop-search input {
  padding-left: 38px;
}

.shop-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.shop-row {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 12px 14px;
  background: var(--app-surface);
  border: 1px solid var(--app-border);
  border-radius: 10px;
  cursor: pointer;
  text-align: left;
  width: 100%;
  color: var(--app-text);
  transition: border-color 0.15s, box-shadow 0.15s;
}

.shop-row:hover {
  border-color: var(--app-accent);
  box-shadow: 0 2px 8px rgba(102,126,234,0.12);
  transform: none;
  background: var(--app-surface);
}

.shop-logo {
  width: 44px; height: 44px;
  border-radius: 8px;
  overflow: hidden;
  background: color-mix(in srgb, var(--app-accent) 12%, var(--app-surface));
  display: flex; align-items: center; justify-content: center;
  flex-shrink: 0;
}

.shop-logo img { width: 100%; height: 100%; object-fit: cover; }

.logo-initial {
  font-size: 18px;
  font-weight: 700;
  color: var(--app-accent);
}

.shop-info { flex: 1; display: flex; flex-direction: column; gap: 2px; }
.shop-info strong { font-size: 15px; }

.shop-info small {
  color: var(--app-muted);
  font-size: 12px !important;
}

.chevron { color: var(--app-muted); flex-shrink: 0; }
.shop-row:hover .chevron { color: var(--app-accent); }

.btn-back {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  background: none;
  border: none;
  color: var(--app-muted);
  font-size: 14px;
  cursor: pointer;
  padding: 0 0 18px;
  font-weight: 500;
  transform: none;
}

.btn-back:hover { color: var(--app-text); background: none; transform: none; }

@media (max-width: 520px) {
  .sv-header {
    align-items: flex-start;
    gap: 12px;
  }

  .top-carousel {
    height: 210px;
  }

  .top-card {
    grid-template-columns: 1fr;
    justify-items: center;
    text-align: center;
  }

  .top-rank {
    justify-self: center;
  }

  .top-info strong {
    white-space: normal;
  }

  .top-stats {
    justify-content: center;
  }
}
</style>
