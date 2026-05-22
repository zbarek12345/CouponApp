<template>
  <div class="shops-view">
    <ShopDetail
      v-if="view === 'detail' && currentShopId"
      :shop-id="currentShopId"
      back-label="All Shops"
      @back="backToBrowser"
      @go-to-coupon="$emit('go-to-coupon', $event)"
      @go-to-receipt="$emit('go-to-receipt', $event)"
    />

    <template v-else-if="view === 'create'">
      <button class="btn-back" @click="backToBrowser">Back</button>
      <CreateShop @navigate="onCreateNavigate" />
    </template>

    <template v-else>
      <div class="sv-header">
        <h2>Shops</h2>
        <div class="sv-header-actions">
          <button class="btn-ghost" @click="loadShops" :disabled="loading">Refresh</button>
          <button @click="view = 'create'">New Shop</button>
        </div>
      </div>

      <div v-if="loading" class="sv-state">
        <div class="spinner"></div>
        <span>Loading...</span>
      </div>

      <div v-else-if="error" class="error">{{ error }}</div>

      <div v-else-if="shops.length === 0" class="sv-state sv-empty">
        <p>No shops yet.</p>
        <button @click="view = 'create'">Create your first shop</button>
      </div>

      <div v-else class="shop-list">
        <button
          v-for="shop in shops"
          :key="shop.shop_id"
          class="shop-row"
          @click="openShop(shop.shop_id)"
        >
          <div class="shop-logo">
            <img
              v-if="shop.logo_base64"
              :src="`data:image/png;base64,${shop.logo_base64}`"
              :alt="shop.shop_name"
            />
            <span v-else class="logo-initial">{{ shop.shop_name.charAt(0).toUpperCase() }}</span>
          </div>
          <div class="shop-info">
            <strong>{{ shop.shop_name }}</strong>
            <small>{{ shop.shop_id }}</small>
          </div>
          <span class="chevron">›</span>
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

defineEmits(['go-to-coupon', 'go-to-receipt'])

const view = ref('browser')
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

const onShopCreated = (shop) => {
  currentShopId.value = shop.shop_id
  view.value = 'detail'
}

const onCreateNavigate = (target, shop) => {
  if (target === 'shop' && shop?.shop_id) {
    onShopCreated(shop)
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
}

.sv-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  padding: 56px 0;
  color: #777;
  text-align: center;
}

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

.shop-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.shop-row {
  display: flex;
  align-items: center;
  gap: 14px;
  width: 100%;
  padding: 12px 14px;
  background: #fff;
  border: 1px solid #e8e8e8;
  border-radius: 8px;
  color: inherit;
  text-align: left;
  transform: none;
}

.shop-row:hover {
  background: #fff;
  border-color: #667eea;
  transform: none;
}

.shop-logo {
  width: 52px;
  height: 52px;
  border-radius: 8px;
  overflow: hidden;
  background: #f0f0ff;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.shop-logo img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.logo-initial {
  font-size: 20px;
  font-weight: 800;
  color: #667eea;
}

.shop-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.shop-info small {
  color: #aaa;
  font-family: monospace;
  font-size: 11px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.chevron {
  color: #aaa;
  font-size: 28px;
  line-height: 1;
}

.btn-back,
.btn-ghost {
  background: transparent;
  color: #555;
  border: 1px solid #ddd;
  transform: none;
}
</style>
