<template>
  <div class="receipt-view">
    <button class="btn-back" type="button" @click="$emit('back')">Back to Receipts</button>

    <div v-if="loading" class="rv-state">
      <div class="spinner"></div>
      <span>Loading receipt...</span>
    </div>

    <div v-else-if="error" class="rv-error">{{ error }}</div>

    <template v-else-if="receipt">
      <section class="receipt-hero">
        <button class="shop-chip" type="button" @click="$emit('go-to-shop', receipt.shop_id)">
          <span>{{ receipt.shop_name }}</span>
          <small>Open shop</small>
        </button>
        <div class="receipt-total">
          <span>Total</span>
          <strong>${{ receipt.total_value.toFixed(2) }}</strong>
        </div>
        <div v-if="receipt.total_discount > 0" class="receipt-discount">
          Discount ${{ receipt.total_discount.toFixed(2) }}
        </div>
      </section>

      <section class="items-card">
        <div class="items-header">
          <h2>Items</h2>
          <span>{{ receipt.entries.length }}</span>
        </div>

        <div v-if="receipt.entries.length === 0" class="rv-state compact">
          No items on this receipt.
        </div>

        <div v-else class="items-list">
          <div v-for="item in receipt.entries" :key="item.entry_id" class="item-row">
            <div class="item-main">
              <strong>{{ item.entry_name }}</strong>
              <small>
                {{ item.entry_quantity }} x ${{ item.entry_cost.toFixed(2) }}
                <template v-if="item.entry_discount > 0">
                  · discount ${{ item.entry_discount.toFixed(2) }}
                </template>
              </small>
            </div>
            <span>${{ itemLineTotal(item).toFixed(2) }}</span>
          </div>
        </div>
      </section>
    </template>
  </div>
</template>

<script setup>
import { onMounted, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const props = defineProps({
  receiptId: { type: String, required: true },
})

defineEmits(['back', 'go-to-shop'])

const receipt = ref(null)
const loading = ref(false)
const error = ref(null)

const loadReceipt = async () => {
  loading.value = true
  error.value = null
  receipt.value = null
  try {
    receipt.value = await invoke('load_receipt_detail', { receiptId: props.receiptId })
  } catch (err) {
    error.value = String(err)
  } finally {
    loading.value = false
  }
}

const itemLineTotal = (item) => item.entry_quantity * item.entry_cost - item.entry_discount

watch(() => props.receiptId, loadReceipt)
onMounted(loadReceipt)
</script>

<style scoped>
.receipt-view {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.btn-back {
  width: fit-content;
  background: transparent;
  color: #555;
  border: 1px solid #ddd;
  transform: none;
}

.receipt-hero,
.items-card {
  background: #fff;
  border: 1px solid #e5e7eb;
  border-radius: 8px;
  padding: 16px;
}

.receipt-hero {
  display: grid;
  grid-template-columns: 1fr auto;
  gap: 12px;
  align-items: center;
}

.shop-chip {
  width: fit-content;
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 10px 12px;
  background: #eef2ff;
  color: #4f46e5;
  border: 0;
  border-radius: 8px;
  text-align: left;
  transform: none;
}

.shop-chip small {
  color: #6366f1;
  font-size: 11px;
}

.receipt-total {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
}

.receipt-total span {
  color: #777;
  font-size: 12px;
}

.receipt-total strong {
  color: #111;
  font-size: 30px;
}

.receipt-discount {
  grid-column: 1 / -1;
  color: #15803d;
  font-weight: 700;
}

.items-header,
.item-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

.items-header {
  margin-bottom: 10px;
}

.items-header h2 {
  margin: 0;
  font-size: 18px;
}

.items-header span {
  color: #777;
  font-size: 13px;
}

.items-list {
  display: flex;
  flex-direction: column;
}

.item-row {
  padding: 12px 0;
  border-top: 1px solid #f0f0f0;
}

.item-main {
  display: flex;
  flex-direction: column;
  gap: 3px;
  min-width: 0;
}

.item-main small {
  color: #777;
}

.item-row > span {
  flex-shrink: 0;
  font-weight: 800;
}

.rv-state {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  padding: 48px 0;
  color: #777;
}

.rv-state.compact {
  padding: 24px 0;
}

.rv-error {
  padding: 14px 16px;
  background: #fff5f5;
  border: 1px solid #fed7d7;
  border-radius: 8px;
  color: #c53030;
}

.spinner {
  width: 26px;
  height: 26px;
  border: 3px solid #eee;
  border-top-color: #667eea;
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

@media (max-width: 640px) {
  .receipt-hero {
    grid-template-columns: 1fr;
  }

  .receipt-total {
    align-items: flex-start;
  }
}
</style>
