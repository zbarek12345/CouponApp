<template>
  <div class="coupons-view">
    <!-- Header + Tab Nav -->
    <div class="view-header">
      <div class="view-title">
        <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">
          <path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z"/>
          <line x1="7" y1="7" x2="7.01" y2="7"/>
        </svg>
        <h1>Coupons</h1>
      </div>

      <nav class="tab-nav">
        <button
          class="tab-btn"
          :class="{ active: activeTab === 'browse' }"
          @click="activeTab = 'browse'"
        >
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/>
            <rect x="14" y="14" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/>
          </svg>
          Browse
        </button>
        <button
          class="tab-btn"
          :class="{ active: activeTab === 'add' }"
          @click="activeTab = 'add'"
        >
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10"/>
            <line x1="12" y1="8" x2="12" y2="16"/>
            <line x1="8" y1="12" x2="16" y2="12"/>
          </svg>
          Add Coupon
        </button>
      </nav>
    </div>

    <!-- Tab panels -->
    <div class="view-body">
      <transition name="tab-slide" mode="out-in">
        <CouponView
          v-if="selectedCouponId"
          key="detail"
          :coupon-id="selectedCouponId"
          back-label="All Coupons"
          @back="selectedCouponId = null"
          @go-to-shop="$emit('go-to-shop', $event)"
        />
        <CouponsBrowseView
          v-else-if="activeTab === 'browse'"
          key="browse"
          :refresh-key="refreshKey"
          @open-coupon="selectedCouponId = $event"
          @go-to-shop="$emit('go-to-shop', $event)"
        />
        <CouponsAddView
          v-else
          key="add"
          @saved="onCouponSaved"
        />
      </transition>
    </div>
  </div>
</template>

<script setup>
import { ref, watch } from 'vue'
import CouponsBrowseView from './CouponsView/CouponsBrowseView.vue'
import CouponsAddView from './CouponsView/CouponsAddView.vue'
import CouponView from './CouponsView/CouponView.vue'

const props = defineProps({
  selectedCouponId: { type: String, default: null },
})

defineEmits(['go-to-shop'])

const activeTab = ref('browse')
const refreshKey = ref(0)
const selectedCouponId = ref(props.selectedCouponId)

const onCouponSaved = () => {
  // Increment triggers a watch in BrowseView → re-fetches from page 1
  refreshKey.value++
  // Switch to browse so the user can see the saved coupon immediately
  activeTab.value = 'browse'
}

watch(
  () => props.selectedCouponId,
  (couponId) => {
    selectedCouponId.value = couponId
    if (couponId) activeTab.value = 'browse'
  },
  { immediate: true }
)
</script>

<style scoped>
.coupons-view {
  display: flex;
  flex-direction: column;
  gap: 0;
  height: 100%;
}

/* ── Header ── */
.view-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 0 10px 0;
  border-bottom: 1px solid var(--app-border);
}

.view-title {
  display: flex;
  align-items: center;
  gap: 10px;
  color: var(--app-accent);
}

.view-title h1 {
  margin: 0;
  font-size: 20px;
  font-weight: 800;
  color: var(--app-text);
  letter-spacing: 0;
  background: none;
  padding: 0;
}

/* ── Tabs ── */
.tab-nav {
  display: flex;
  gap: 4px;
  padding-bottom: 0;
}

.tab-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 9px 16px;
  border: none;
  border-bottom: 2px solid transparent;
  background: none;
  cursor: pointer;
  font-size: 14px;
  font-weight: 600;
  color: var(--app-muted);
  border-radius: 6px 6px 0 0;
  transition: color 0.15s, border-color 0.15s, background 0.15s;
  margin-bottom: -1px; /* sit on the border */
}

.tab-btn:hover {
  color: var(--app-text);
  background: var(--app-surface-alt);
}

.tab-btn.active {
  color: var(--app-accent);
  border-bottom-color: var(--app-accent);
  background: var(--app-surface-alt);
}

/* ── Body ── */
.view-body {
  flex: 1;
  overflow-y: auto;
  padding: 15px 0;
}

/* ── Transition ── */
.tab-slide-enter-active,
.tab-slide-leave-active {
  transition: opacity 0.18s ease, transform 0.18s ease;
}

.tab-slide-enter-from {
  opacity: 0;
  transform: translateX(10px);
}

.tab-slide-leave-to {
  opacity: 0;
  transform: translateX(-10px);
}
</style>
