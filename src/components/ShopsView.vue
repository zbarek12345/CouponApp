<template>
  <div>
    <div class="card">
      <h2>Create New Shop</h2>
      <div class="form-group">
        <input v-model="newShop.name" placeholder="Shop Name" />
        <input v-model="newShop.logo" placeholder="Logo URL (optional)" />
        <button @click="createShop" :disabled="!newShop.name">Create Shop</button>
      </div>
    </div>

    <div class="card">
      <h2>All Shops</h2>
      <button @click="loadShops" :disabled="loading">Refresh</button>
      
      <div v-if="loading" class="loading">Loading...</div>
      <div v-else-if="error" class="error">{{ error }}</div>
      <div v-else>
        <div v-for="shop in shops" :key="shop.shop_id" class="shop-item">
          <div class="shop-info">
            <strong>{{ shop.shop_name }}</strong>
            <span v-if="shop.shop_logo" class="logo">{{ shop.shop_logo }}</span>
            <small class="id">{{ shop.shop_id }}</small>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const shops = ref([])
const loading = ref(false)
const error = ref(null)
const newShop = ref({ name: '', logo: '' })

const loadShops = async () => {
  loading.value = true
  error.value = null
  try {
    shops.value = await invoke('load_shops')
  } catch (err) {
    error.value = err
    console.error(err)
  } finally {
    loading.value = false
  }
}

const createShop = async () => {
  loading.value = true
  error.value = null
  try {
    await invoke('create_shop', { 
      request: { 
        name: newShop.value.name, 
        logo: newShop.value.logo || null 
      } 
    })
    await loadShops()
    newShop.value = { name: '', logo: '' }
  } catch (err) {
    error.value = err
    console.error(err)
  } finally {
    loading.value = false
  }
}

// Load initial data
loadShops()
</script>

<style scoped>
.form-group {
  display: flex;
  gap: 10px;
  margin-top: 10px;
}

.shop-item {
  padding: 10px;
  border-bottom: 1px solid #eee;
}

.shop-info {
  display: flex;
  align-items: center;
  gap: 10px;
}

.id {
  color: #999;
  font-size: 12px;
}

.logo {
  color: #667eea;
  font-size: 12px;
}

.loading {
  text-align: center;
  padding: 20px;
  color: #999;
}
</style>