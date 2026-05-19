<template>
  <div class="app">
    <h1>📱 Coupon & Receipt Manager</h1>
    
    <div class="tabs">
      <button 
        v-for="tab in tabs" 
        :key="tab.id"
        :class="{ active: activeTab === tab.id }"
        @click="activeTab = tab.id"
      >
        {{ tab.name }}
      </button>
    </div>

    <div class="tab-content">
      <ShopsView v-if="activeTab === 'shops'" />
      <CouponsView v-if="activeTab === 'coupons'" />
      <ReceiptsView v-if="activeTab === 'receipts'" />
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import ShopsView from './components/ShopsView.vue'
import CouponsView from './components/CouponsView.vue'
import ReceiptsView from './components/ReceiptsView.vue'

const activeTab = ref('shops')
const tabs = [
  { id: 'shops', name: '🏪 Shops' },
  { id: 'coupons', name: '🎫 Coupons' },
  { id: 'receipts', name: '🧾 Receipts' }
]
</script>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  background: #f5f5f5;
  padding: 20px;
}

.app {
  max-width: 1200px;
  margin: 0 auto;
  background: white;
  border-radius: 12px;
  box-shadow: 0 2px 10px rgba(0,0,0,0.1);
  overflow: hidden;
}

h1 {
  padding: 20px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  margin: 0;
}

.tabs {
  display: flex;
  border-bottom: 2px solid #e0e0e0;
  background: #fafafa;
}

.tabs button {
  flex: 1;
  padding: 15px;
  background: none;
  border: none;
  font-size: 16px;
  cursor: pointer;
  transition: all 0.3s;
  font-weight: 500;
}

.tabs button:hover {
  background: #f0f0f0;
}

.tabs button.active {
  background: white;
  color: #667eea;
  border-bottom: 3px solid #667eea;
}

.tab-content {
  padding: 20px;
}

button {
  background: #667eea;
  color: white;
  border: none;
  padding: 8px 16px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
  transition: all 0.3s;
}

button:hover {
  background: #5a67d8;
  transform: translateY(-1px);
}

button:disabled {
  background: #ccc;
  cursor: not-allowed;
  transform: none;
}

input, textarea, select {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid #ddd;
  border-radius: 6px;
  font-size: 14px;
}

.card {
  background: white;
  border: 1px solid #e0e0e0;
  border-radius: 8px;
  padding: 15px;
  margin-bottom: 15px;
}

.error {
  background: #fee;
  color: #c33;
  padding: 10px;
  border-radius: 6px;
  margin-bottom: 15px;
}

.success {
  background: #efe;
  color: #3c3;
  padding: 10px;
  border-radius: 6px;
  margin-bottom: 15px;
}

.grid {
  display: grid;
  gap: 15px;
}
</style>