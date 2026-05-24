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
          <button class="btn-ghost icon-btn" @click="displayMode = displayMode === 'list' ? 'carousel' : 'list'" title="Toggle view mode">
            <svg v-if="displayMode === 'list'" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/></svg>
            <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="8" y1="6" x2="21" y2="6"/><line x1="8" y1="12" x2="21" y2="12"/><line x1="8" y1="18" x2="21" y2="18"/><line x1="3" y1="6" x2="3.01" y2="6"/><line x1="3" y1="12" x2="3.01" y2="12"/><line x1="3" y1="18" x2="3.01" y2="18"/></svg>
          </button>
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

      <div
        v-else
        :class="['shop-list', displayMode === 'carousel' ? 'shop-list-carousel' : '']"
        @touchstart.stop
        @touchend.stop
        @touchmove.stop
      >
        <button
          v-for="shop in shops"
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
          </div>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="chevron"><polyline points="9 18 15 12 9 6"/></svg>
        </button>
      </div>
    </template>
  </div>
</template>

<script setup>
import { onMounted, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import ShopDetail from './ShopsView/ShopDetail.vue'
import CreateShop from './ShopsView/CreateShop.vue'

const props = defineProps({
  selectedShopId: { type: String, default: null },
})

const emit = defineEmits(['go-to-coupon', 'go-to-receipt'])

const view = ref('browser')
const displayMode = ref('list')
const shops = ref([])
const currentShopId = ref(null)
const loading = ref(false)
const error = ref(null)

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
  color: #667eea;
  border: 1px solid transparent;
}

.btn-ghost:hover {
  background: #f0f0ff;
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
  color: #999;
  text-align: center;
}

.sv-empty p { margin: 0; }

.spinner {
  width: 28px;
  height: 28px;
  border: 3px solid #eee;
  border-top-color: #667eea;
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.shop-list { display: flex; flex-direction: column; gap: 8px; }

.shop-list-carousel {
  flex-direction: row;
  flex-wrap: nowrap;
  overflow-x: auto;
  overflow-y: hidden;
  gap: 12px;
  padding-bottom: 8px;
  scroll-snap-type: x mandatory;
  overscroll-behavior-x: contain;
  touch-action: pan-x;
  scrollbar-width: none;
}

.shop-list-carousel::-webkit-scrollbar {
  display: none;
}

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

.shop-list-carousel .shop-row {
  flex: 0 0 82%;
  max-width: 300px;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 18px 16px 16px;
  scroll-snap-align: start;
}

.shop-row:hover {
  border-color: #667eea;
  box-shadow: 0 2px 8px rgba(102,126,234,0.12);
  transform: none;
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

.shop-list-carousel .shop-logo {
  width: 104px;
  height: 104px;
  border-radius: 18px;
}

.shop-list-carousel .logo-initial {
  font-size: 34px;
}

.shop-info { flex: 1; display: flex; flex-direction: column; gap: 2px; }
.shop-info strong { font-size: 15px; }

.shop-list-carousel .shop-info {
  flex: 0;
  width: 100%;
  gap: 0;
  align-items: center;
  text-align: center;
}

.shop-list-carousel .shop-info strong {
  font-size: 17px;
  line-height: 1.2;
}

.chevron { color: #ccc; flex-shrink: 0; }
.shop-row:hover .chevron { color: #667eea; }

.shop-list-carousel .chevron {
  display: none;
}

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
</style>
