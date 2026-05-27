<template>
  <div class="receipts-view">
    <!-- Header + Tab Nav -->
    <div class="view-header">
      <div class="view-title">
        <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">
          <path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>
        </svg>
        <h1>Receipts</h1>
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
          Add Receipt
        </button>
      </nav>
    </div>

    <!-- Tab panels -->
    <div class="view-body">
      <transition name="tab-slide" mode="out-in">
        <ReceiptsBrowseView
          v-if="activeTab === 'browse'"
          key="browse"
          @go-to-shop="$emit('go-to-shop', $event)"
        />
        <ReceiptsAddView
          v-else
          key="add"
          @saved="activeTab = 'browse'"
        />
      </transition>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import ReceiptsBrowseView from './ReceiptsView/ReceiptsBrowseView.vue'
import ReceiptsAddView from './ReceiptsView/ReceiptsAddView.vue'

defineEmits(['go-to-shop'])

const activeTab = ref('browse')
</script>

<style scoped>
.receipts-view {
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
  padding: 15px;
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
  color: var(--app-muted);
  border: 1px solid var(--app-border);
}

.btn-ghost:hover {
  background: var(--app-surface-alt);
  color: var(--app-text);
  transform: none;
}
</style>
