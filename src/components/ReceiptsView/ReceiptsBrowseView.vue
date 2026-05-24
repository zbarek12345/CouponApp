<template>
  <div>
    <ReceiptView
      v-if="selectedReceipt"
      :receipt-id="selectedReceipt"
      @back="selectedReceipt = null"
      @go-to-shop="$emit('go-to-shop', $event)"
    />

    <template v-else>
      <div class="pagination">
        <button @click="loadReceipts(0)" :disabled="loading">Refresh</button>
        <button @click="loadReceipts(offset - limit)" :disabled="offset === 0">Previous</button>
        <button @click="loadReceipts(offset + limit)" :disabled="!hasMore">Next</button>
      </div>

      <div v-if="loading" class="loading">Loading...</div>
      <div v-else-if="error" class="error">{{ error }}</div>
      <div v-else>
        <div v-for="receipt in receipts" :key="receipt.receipt_id" class="receipt-item-summary">
          <div class="receipt-header" @click="viewDetail(receipt.receipt_id)">
            <strong>{{ receipt.shop_name }}</strong>
            <span class="total">${{ receipt.total_value.toFixed(2) }}</span>
          </div>
        </div>
      </div>
    </template>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import ReceiptView from '../ReceiptView.vue'

defineEmits(['go-to-shop'])

const receipts = ref([])
const loading = ref(false)
const error = ref(null)
const offset = ref(0)
const limit = ref(10)
const hasMore = ref(false)
const selectedReceipt = ref(null)

const loadReceipts = async (newOffset = 0) => {
  loading.value = true
  error.value = null
  try {
    const result = await invoke('load_receipts', { offset: newOffset, limit: limit.value })
    receipts.value = result.items
    offset.value = result.offset
    hasMore.value = result.items.length === limit.value
  } catch (err) {
    error.value = err
    console.error(err)
  } finally {
    loading.value = false
  }
}

const viewDetail = async (receiptId) => {
  selectedReceipt.value = receiptId
}

onMounted(() => loadReceipts(0))
</script>

<style scoped>
.pagination {
  display: flex;
  gap: 10px;
  align-items: center;
  margin-bottom: 20px;
}

.receipt-item-summary {
  border: 1px solid #eee;
  margin-bottom: 10px;
  border-radius: 8px;
  overflow: hidden;
}

.receipt-header {
  padding: 12px;
  background: #f9f9f9;
  cursor: pointer;
  display: flex;
  justify-content: space-between;
  transition: background 0.3s;
}

.receipt-header:hover {
  background: #f0f0f0;
}

.total {
  font-weight: bold;
  color: #667eea;
}
</style>
