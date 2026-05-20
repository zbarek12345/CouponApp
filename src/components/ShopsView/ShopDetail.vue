<template>
  <div class="detail-view">
    <button class="back-btn" @click="$emit('navigate', 'browser')">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="15 18 9 12 15 6"/></svg>
      All Shops
    </button>

    <!-- Shop Header -->
    <div class="shop-header">
      <div class="shop-avatar">
        <img
          v-if="shop.logo_base64"
          :src="`data:image/png;base64,${shop.logo_base64}`"
          :alt="shop.shop_name"
        />
        <span v-else class="avatar-fallback">{{ shop.shop_name.charAt(0).toUpperCase() }}</span>
      </div>
      <div class="shop-title">
        <h1>{{ shop.shop_name }}</h1>
        <code class="shop-id">{{ shop.shop_id }}</code>
      </div>
    </div>

    <!-- Tabs -->
    <div class="tabs">
      <button
        v-for="tab in tabs"
        :key="tab.id"
        class="tab-btn"
        :class="{ active: activeTab === tab.id }"
        @click="activeTab = tab.id"
      >
        <component :is="tab.icon" />
        {{ tab.label }}
        <span class="tab-badge">{{ tab.count }}</span>
      </button>
    </div>

    <!-- Coupons Tab -->
    <div v-if="activeTab === 'coupons'" class="tab-content">
      <div class="tab-toolbar">
        <h2>Coupons</h2>
        <button class="btn-primary" disabled title="Coming soon">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
          Add Coupon
        </button>
      </div>

      <div class="placeholder-grid">
        <div v-for="n in 3" :key="n" class="coupon-placeholder placeholder-shimmer">
          <div class="coupon-ph-left">
            <div class="ph-bar ph-bar--title"></div>
            <div class="ph-bar ph-bar--subtitle"></div>
          </div>
          <div class="coupon-ph-right">
            <div class="ph-chip"></div>
          </div>
        </div>
      </div>

      <div class="empty-notice">
        <svg width="36" height="36" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1"><path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z"/><line x1="7" y1="7" x2="7.01" y2="7"/></svg>
        <p>Coupon management coming soon.</p>
      </div>
    </div>

    <!-- Receipts Tab -->
    <div v-if="activeTab === 'receipts'" class="tab-content">
      <div class="tab-toolbar">
        <h2>Receipts</h2>
        <button class="btn-primary" disabled title="Coming soon">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
          Add Receipt
        </button>
      </div>

      <div class="placeholder-grid">
        <div v-for="n in 4" :key="n" class="receipt-placeholder placeholder-shimmer">
          <div class="ph-bar ph-bar--date"></div>
          <div class="ph-bar ph-bar--title"></div>
          <div class="ph-bar ph-bar--amount"></div>
        </div>
      </div>

      <div class="empty-notice">
        <svg width="36" height="36" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/><polyline points="10 9 9 9 8 9"/></svg>
        <p>Receipt tracking coming soon.</p>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue'

const props = defineProps({
  shop: {
    type: Object,
    required: true,
  },
})

defineEmits(['navigate'])

const activeTab = ref('coupons')

const tabs = [
  { id: 'coupons', label: 'Coupons', count: 0 },
  { id: 'receipts', label: 'Receipts', count: 0 },
]
</script>

<style scoped>
.detail-view {
  padding: 32px;
  max-width: 900px;
  margin: 0 auto;
}

.back-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  background: none;
  border: none;
  color: #666;
  font-size: 14px;
  cursor: pointer;
  padding: 0;
  margin-bottom: 28px;
  transition: color 0.15s;
}
.back-btn:hover { color: #111; }

/* Shop header */
.shop-header {
  display: flex;
  align-items: center;
  gap: 20px;
  margin-bottom: 32px;
}

.shop-avatar {
  width: 72px;
  height: 72px;
  border-radius: 16px;
  overflow: hidden;
  background: #f0f0ff;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid #e0e0ff;
}
.shop-avatar img { width: 100%; height: 100%; object-fit: cover; }
.avatar-fallback {
  font-size: 28px;
  font-weight: 700;
  color: #4f46e5;
}

.shop-title h1 {
  font-size: 26px;
  font-weight: 700;
  margin: 0 0 6px;
  letter-spacing: -0.5px;
}
.shop-id {
  font-size: 11px;
  color: #bbb;
  background: #f7f7f7;
  padding: 3px 8px;
  border-radius: 4px;
  font-family: monospace;
}

/* Tabs */
.tabs {
  display: flex;
  gap: 4px;
  border-bottom: 2px solid #eee;
  margin-bottom: 28px;
}

.tab-btn {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  padding: 10px 16px;
  background: none;
  border: none;
  border-bottom: 2px solid transparent;
  margin-bottom: -2px;
  font-size: 14px;
  font-weight: 500;
  color: #888;
  cursor: pointer;
  transition: color 0.15s, border-color 0.15s;
}
.tab-btn:hover { color: #333; }
.tab-btn.active {
  color: #4f46e5;
  border-bottom-color: #4f46e5;
}

.tab-badge {
  background: #f0f0f0;
  color: #888;
  font-size: 11px;
  font-weight: 600;
  padding: 1px 6px;
  border-radius: 10px;
}
.tab-btn.active .tab-badge {
  background: #ede9fe;
  color: #4f46e5;
}

/* Tab toolbar */
.tab-content { }
.tab-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 20px;
}
.tab-toolbar h2 {
  font-size: 18px;
  font-weight: 600;
  margin: 0;
}

.btn-primary {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  background: #4f46e5;
  color: #fff;
  border: none;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.15s;
}
.btn-primary:hover:not(:disabled) { background: #4338ca; }
.btn-primary:disabled { opacity: 0.4; cursor: not-allowed; }

/* Placeholder shimmer */
.placeholder-grid {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin-bottom: 28px;
}

@keyframes shimmer {
  0% { background-position: -600px 0; }
  100% { background-position: 600px 0; }
}

.placeholder-shimmer {
  background: linear-gradient(90deg, #f5f5f5 25%, #ebebeb 50%, #f5f5f5 75%);
  background-size: 600px 100%;
  animation: shimmer 1.4s infinite linear;
  border-radius: 10px;
}

/* Coupon placeholder */
.coupon-placeholder {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 18px 20px;
  border: 1px solid #eee;
}
.coupon-ph-left { display: flex; flex-direction: column; gap: 8px; flex: 1; }
.coupon-ph-right { flex-shrink: 0; }

/* Receipt placeholder */
.receipt-placeholder {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 16px 20px;
  border: 1px solid #eee;
}

/* Placeholder bars */
.ph-bar {
  border-radius: 4px;
  background: rgba(0,0,0,0.06);
}
.ph-bar--title { height: 14px; width: 160px; }
.ph-bar--subtitle { height: 11px; width: 100px; }
.ph-bar--date { height: 11px; width: 70px; }
.ph-bar--amount { height: 14px; width: 60px; }

.ph-chip {
  width: 56px;
  height: 24px;
  border-radius: 12px;
  background: rgba(0,0,0,0.06);
}

/* Empty notice */
.empty-notice {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  padding: 32px;
  color: #bbb;
  border: 2px dashed #eee;
  border-radius: 12px;
  text-align: center;
}
.empty-notice p { margin: 0; font-size: 14px; color: #aaa; }
</style>