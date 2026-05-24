<template>
  <div class="receipts-view">
    <div class="receipts-view-header">
      <h2>Receipts</h2>
      <button v-if="activeView === 'list'" @click="activeView = 'add'">+ Add Receipt</button>
      <button v-else class="btn-ghost" @click="activeView = 'list'">Back to list</button>
    </div>

    <ReceiptsBrowseView v-if="activeView === 'list'" @go-to-shop="$emit('go-to-shop', $event)" />
    <ReceiptsAddView v-else @saved="activeView = 'list'" />
  </div>
</template>

<script setup>
import { ref } from 'vue'
import ReceiptsBrowseView from './ReceiptsView/ReceiptsBrowseView.vue'
import ReceiptsAddView from './ReceiptsView/ReceiptsAddView.vue'

defineEmits(['go-to-shop'])

const activeView = ref('list')
</script>

<style scoped>
.receipts-view {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.receipts-view-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.receipts-view-header h2 {
  margin: 0;
}

.btn-ghost {
  background: transparent;
  color: #555;
  border: 1px solid #ddd;
}

.btn-ghost:hover {
  background: #f5f5f5;
  transform: none;
}
</style>
