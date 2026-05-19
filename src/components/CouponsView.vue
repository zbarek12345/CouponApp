<template>
  <div>
    <div class="card">
      <h2>Scan Coupon</h2>
      <div class="scan-section">
        <textarea v-model="mockImageData" placeholder="Paste mock image data or base64..."></textarea>
        <button @click="scanCoupon" :disabled="scanning">Scan Image</button>
      </div>

      <div v-if="preview" class="preview">
        <h3>Scan Results</h3>
        <div class="candidates">
          <div 
            v-for="(candidate, idx) in preview.candidates" 
            :key="idx"
            class="candidate"
            :class="{ selected: selectedCandidate === idx }"
            @click="selectedCandidate = idx"
          >
            <strong>{{ candidate.code_type }}</strong>
            <span>{{ candidate.code_value }}</span>
            <small>Confidence: {{ (candidate.confidence * 100).toFixed(1) }}%</small>
          </div>
        </div>
        
        <div class="form-group">
          <input v-model="description" placeholder="Description" />
          <select v-model="selectedShopId">
            <option value="">Select Shop</option>
            <option v-for="shop in shops" :key="shop.shop_id" :value="shop.shop_id">
              {{ shop.shop_name }}
            </option>
          </select>
          <button @click="saveCoupon" :disabled="saving">Save Coupon</button>
        </div>
      </div>
    </div>

    <div class="card">
      <h2>Coupons List</h2>
      <div class="pagination">
        <button @click="loadCoupons(0)" :disabled="loading">Refresh</button>
        <span>Page: {{ Math.floor(offset / limit) + 1 }}</span>
        <button @click="loadCoupons(offset - limit)" :disabled="offset === 0">Previous</button>
        <button @click="loadCoupons(offset + limit)" :disabled="!hasMore">Next</button>
      </div>
      
      <div v-if="loading" class="loading">Loading...</div>
      <div v-else-if="error" class="error">{{ error }}</div>
      <div v-else>
        <div v-for="coupon in coupons" :key="coupon.coupon_id" class="coupon-item">
          <div class="coupon-header">
            <strong>{{ coupon.shop_name }}</strong>
            <span class="badge">{{ coupon.code_type }}</span>
          </div>
          <div class="coupon-body">
            <span class="code">{{ coupon.code_value }}</span>
            <span class="description">{{ coupon.description || 'No description' }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const shops = ref([])
const coupons = ref([])
const loading = ref(false)
const error = ref(null)
const offset = ref(0)
const limit = ref(10)
const hasMore = ref(false)

// Scan state
const mockImageData = ref('')
const scanning = ref(false)
const saving = ref(false)
const preview = ref(null)
const selectedCandidate = ref(0)
const description = ref('')
const selectedShopId = ref('')

const loadShops = async () => {
  try {
    shops.value = await invoke('load_shops')
  } catch (err) {
    console.error(err)
  }
}

const loadCoupons = async (newOffset = 0) => {
  loading.value = true
  error.value = null
  try {
    const result = await invoke('load_coupons', { offset: newOffset, limit: limit.value })
    coupons.value = result.items
    offset.value = result.offset
    hasMore.value = result.items.length === limit.value
  } catch (err) {
    error.value = err
    console.error(err)
  } finally {
    loading.value = false
  }
}

const scanCoupon = async () => {
  scanning.value = true
  error.value = null
  try {
    const result = await invoke('scan_coupon_image', {
      request: { image_base64: mockImageData.value || 'mock-image-data' }
    })
    preview.value = result
    selectedCandidate.value = result.best_index
    await loadShops()
  } catch (err) {
    error.value = err
    console.error(err)
  } finally {
    scanning.value = false
  }
}

const saveCoupon = async () => {
  if (!selectedShopId.value) {
    error.value = 'Please select a shop'
    return
  }
  
  saving.value = true
  error.value = null
  try {
    await invoke('save_coupon', {
      request: {
        candidates: preview.value.candidates,
        selected_candidate_index: selectedCandidate.value,
        description: description.value,
        shop_id: selectedShopId.value
      }
    })
    await loadCoupons(0)
    // Reset form
    preview.value = null
    description.value = ''
    selectedShopId.value = ''
    mockImageData.value = ''
  } catch (err) {
    error.value = err
    console.error(err)
  } finally {
    saving.value = false
  }
}

loadCoupons(0)
</script>

<style scoped>
.scan-section {
  display: flex;
  gap: 10px;
  margin-bottom: 20px;
}

.scan-section textarea {
  flex: 1;
  min-height: 100px;
}

.preview {
  margin-top: 20px;
  padding-top: 20px;
  border-top: 1px solid #eee;
}

.candidates {
  margin: 15px 0;
}

.candidate {
  padding: 10px;
  margin-bottom: 8px;
  border: 1px solid #ddd;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.3s;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.candidate:hover {
  background: #f5f5f5;
}

.candidate.selected {
  background: #e8eaf6;
  border-color: #667eea;
}

.coupon-item {
  padding: 12px;
  margin-bottom: 10px;
  border: 1px solid #eee;
  border-radius: 8px;
}

.coupon-header {
  display: flex;
  justify-content: space-between;
  margin-bottom: 8px;
}

.badge {
  background: #667eea;
  color: white;
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 12px;
}

.code {
  font-family: monospace;
  background: #f5f5f5;
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 12px;
}

.description {
  margin-left: 10px;
  color: #666;
}

.pagination {
  display: flex;
  gap: 10px;
  align-items: center;
  margin-bottom: 20px;
}
</style>