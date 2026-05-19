<template>
  <div class="add-view">
    <!-- Upload Zone (collapses after scan) -->
    <div class="upload-card" :class="{ 'has-image': imagePreview, compact: !!preview }">
      <div
        class="drop-zone"
        :class="{ 'drag-over': isDragging, compact: !!preview }"
        @click="!preview && triggerFileInput()"
        @drop.prevent="handleDrop"
        @dragover.prevent="isDragging = true"
        @dragleave.prevent="isDragging = false"
      >
        <input
          type="file"
          ref="fileInput"
          @change="handleFileSelect"
          accept="image/*"
          style="display: none"
        />

        <!-- Empty placeholder -->
        <transition name="fade" mode="out-in">
          <div v-if="!imagePreview" key="placeholder" class="drop-placeholder">
            <div class="upload-icon">
              <svg width="36" height="36" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
                <polyline points="17 8 12 3 7 8"/>
                <line x1="12" y1="3" x2="12" y2="15"/>
              </svg>
            </div>
            <p class="drop-label">Drop coupon image here</p>
            <span class="drop-hint">or click to browse — JPG, PNG, WebP</span>
          </div>

          <!-- Image + SVG overlay after scan; plain preview before scan -->
          <div v-else key="preview" class="preview-wrapper">

            <!-- Visual code selector: shown after scan results arrive -->
            <div v-if="preview" class="image-overlay-root" ref="overlayRoot">
              <img
                :src="imagePreview"
                class="overlay-img"
                ref="overlayImg"
                @load="onImageLoad"
                draggable="false"
              />
              <!-- SVG drawn in image-native coordinates, scaled via viewBox -->
              <svg
                v-if="imgNaturalW && imgNaturalH"
                class="overlay-svg"
                :viewBox="`0 0 ${imgNaturalW} ${imgNaturalH}`"
                xmlns="http://www.w3.org/2000/svg"
                preserveAspectRatio="xMidYMid meet"
              >
                <defs>
                  <filter id="glow">
                    <feGaussianBlur stdDeviation="4" result="blur"/>
                    <feMerge><feMergeNode in="blur"/><feMergeNode in="SourceGraphic"/></feMerge>
                  </filter>
                </defs>

                <g
                  v-for="(candidate, idx) in preview.candidates"
                  :key="idx"
                  class="code-region"
                  :class="{
                    'is-selected': selectedCandidate === idx,
                    'is-hovered': hoveredCandidate === idx,
                  }"
                  @click.stop="selectedCandidate = idx"
                  @mouseenter="hoveredCandidate = idx"
                  @mouseleave="hoveredCandidate = null"
                  style="cursor: pointer"
                >
                  <!-- Polygon (≥3 points) -->
                  <polygon
                    v-if="candidate.bounds.length >= 3"
                    :points="toSvgPoints(candidate.bounds)"
                    :fill="regionFill(idx)"
                    :stroke="regionStroke(idx)"
                    stroke-width="6"
                    stroke-linejoin="round"
                    :filter="(selectedCandidate === idx || hoveredCandidate === idx) ? 'url(#glow)' : ''"
                    style="transition: fill 0.15s, stroke 0.15s"
                  />

                  <!-- Line (2 points) -->
                  <line
                    v-else-if="candidate.bounds.length === 2"
                    :x1="candidate.bounds[0].x" :y1="candidate.bounds[0].y"
                    :x2="candidate.bounds[1].x" :y2="candidate.bounds[1].y"
                    :stroke="regionStroke(idx)"
                    stroke-width="8"
                    stroke-linecap="round"
                    :filter="(selectedCandidate === idx || hoveredCandidate === idx) ? 'url(#glow)' : ''"
                    style="transition: stroke 0.15s"
                  />

                  <!-- Label chip anchored to the first bound point -->
                  <g
                    v-if="candidate.bounds.length > 0"
                    :transform="`translate(${labelAnchor(candidate.bounds).x}, ${labelAnchor(candidate.bounds).y})`"
                  >
                    <!-- chip background -->
                    <rect
                      :x="-labelPad"
                      :y="-(labelH - labelPad)"
                      :width="labelTextWidth(idx)"
                      :height="labelH"
                      rx="14"
                      :fill="regionStroke(idx)"
                    />
                    <!-- index number -->
                    <text
                      x="0"
                      y="0"
                      dominant-baseline="auto"
                      text-anchor="start"
                      :font-size="labelFontSize"
                      font-family="monospace"
                      font-weight="700"
                      fill="white"
                      style="user-select: none; pointer-events: none"
                    >{{ idx + 1 }} · {{ candidate.code_type }}</text>
                  </g>
                </g>
              </svg>
            </div>

            <!-- Plain image preview (before scan) -->
            <img v-else :src="imagePreview" class="preview-img" draggable="false" />

            <button class="clear-btn" @click.stop="clearImage">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <line x1="18" y1="6" x2="6" y2="18"/>
                <line x1="6" y1="6" x2="18" y2="18"/>
              </svg>
            </button>
          </div>
        </transition>
      </div>

      <!-- Paste base64 fallback (hidden once preview is loaded) -->
      <div v-if="!imagePreview" class="alt-input">
        <span class="alt-label">or paste base64</span>
        <textarea
          v-model="pastedBase64"
          placeholder="data:image/... or raw base64"
          rows="2"
          @input="onBase64Paste"
        />
      </div>

      <button
        class="scan-btn"
        :class="{ loading: scanning }"
        @click="scanCoupon"
        :disabled="scanning || !effectiveBase64"
      >
        <span v-if="scanning" class="btn-spinner" />
        <svg v-else width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="23 4 23 10 17 10"/>
          <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/>
        </svg>
        {{ scanning ? 'Scanning…' : 'Scan Image' }}
      </button>
    </div>

    <!-- Error Banner -->
    <transition name="slide-down">
      <div v-if="error" class="error-banner">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"/>
        </svg>
        {{ error }}
        <button @click="error = null" class="dismiss-btn">✕</button>
      </div>
    </transition>

    <!-- Code Picker + Assignment -->
    <transition name="slide-up">
      <div v-if="preview" class="results-section">
        <div class="section-header">
          <h3>Detected Codes</h3>
          <span class="badge">{{ preview.candidates.length }} found</span>
          <span class="hint-text">Click a region in the image or a card below</span>
        </div>

        <!-- Candidate cards (sync with visual selection) -->
        <div class="candidates-grid">
          <div
            v-for="(candidate, idx) in preview.candidates"
            :key="idx"
            class="candidate-card"
            :class="{ selected: selectedCandidate === idx, hovered: hoveredCandidate === idx }"
            @click="selectedCandidate = idx"
            @mouseenter="hoveredCandidate = idx"
            @mouseleave="hoveredCandidate = null"
          >
            <!-- Colour swatch matching SVG region -->
            <div class="card-swatch" :style="{ background: regionStroke(idx) }" />

            <div class="candidate-meta">
              <span class="index-badge" :style="{ background: regionStroke(idx) }">{{ idx + 1 }}</span>
              <span class="type-pill">{{ candidate.code_type }}</span>
              <span class="conf-pill" :class="confidenceClass(candidate.confidence)">
                {{ (candidate.confidence * 100).toFixed(1) }}%
              </span>
            </div>

            <div class="candidate-value">{{ candidate.code_value }}</div>

            <div v-if="selectedCandidate === idx" class="check-mark">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
                <polyline points="20 6 9 17 4 12"/>
              </svg>
              Selected
            </div>
          </div>
        </div>

        <!-- Assignment Form -->
        <div class="assign-card">
          <h3 class="assign-title">Assign Coupon</h3>

          <div class="field-row">
            <div class="field">
              <label>Description <span class="optional">(optional)</span></label>
              <input
                v-model="description"
                type="text"
                placeholder="e.g. 10% off electronics"
                class="text-input"
              />
            </div>

            <div class="field">
              <label>Shop <span class="required">*</span></label>
              <div class="select-wrapper">
                <select v-model="selectedShopId" class="select-input">
                  <option value="">Select a shop…</option>
                  <option v-for="shop in shops" :key="shop.shop_id" :value="shop.shop_id">
                    {{ shop.shop_name }}
                  </option>
                </select>
                <svg class="select-arrow" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <polyline points="6 9 12 15 18 9"/>
                </svg>
              </div>
            </div>
          </div>

          <button
            class="save-btn"
            :class="{ loading: saving }"
            @click="saveCoupon"
            :disabled="saving || !selectedShopId"
          >
            <span v-if="saving" class="btn-spinner" />
            <svg v-else width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/>
              <polyline points="17 21 17 13 7 13 7 21"/>
              <polyline points="7 3 7 8 15 8"/>
            </svg>
            {{ saving ? 'Saving…' : 'Save Coupon' }}
          </button>
        </div>
      </div>
    </transition>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const emit = defineEmits(['saved'])

// — File / image state —
const fileInput    = ref(null)
const selectedFile = ref(null)
const imagePreview = ref(null)
const pastedBase64 = ref('')
const isDragging   = ref(false)

// — Overlay state —
const overlayImg    = ref(null)
const imgNaturalW   = ref(0)
const imgNaturalH   = ref(0)
const hoveredCandidate = ref(null)

// — Scan / save state —
const scanning        = ref(false)
const saving          = ref(false)
const error           = ref(null)
const preview         = ref(null)
const selectedCandidate = ref(0)
const description     = ref('')
const selectedShopId  = ref('')
const shops           = ref([])

// ── Colour palette for up to N codes ──────────────────────────────────────────
// Perceptually distinct, works on both light and dark image regions
const REGION_COLORS = [
  '#6366f1', // indigo
  '#f59e0b', // amber
  '#10b981', // emerald
  '#ef4444', // red
  '#8b5cf6', // violet
  '#06b6d4', // cyan
  '#f97316', // orange
  '#ec4899', // pink
]

const regionStroke = (idx) => REGION_COLORS[idx % REGION_COLORS.length]
const regionFill   = (idx) => {
  const base = REGION_COLORS[idx % REGION_COLORS.length]
  const active = selectedCandidate.value === idx || hoveredCandidate.value === idx
  // Return fill with opacity encoded as hex suffix
  return base + (active ? '40' : '18')
}

// ── SVG label sizing (in image-native px, will scale with viewBox) ────────────
const labelFontSize = 28   // px in image space — looks ~12px when rendered
const labelH        = 40
const labelPad      = 12
const labelCharW    = 17   // monospace estimate

const labelTextWidth = (idx) => {
  const c = preview.value?.candidates?.[idx]
  if (!c) return 80
  const text = `${idx + 1} · ${c.code_type}`
  return text.length * labelCharW + labelPad * 2
}

// Anchor: slightly above the top-left bounding point
const labelAnchor = (bounds) => {
  // Use the point with smallest y (topmost)
  const top = bounds.reduce((a, b) => (b.y < a.y ? b : a), bounds[0])
  return { x: top.x, y: Math.max(top.y - 10, labelH) }
}

// ── Helpers ───────────────────────────────────────────────────────────────────
const toSvgPoints = (bounds) =>
  bounds.map((p) => `${p.x},${p.y}`).join(' ')

const confidenceClass = (conf) => {
  if (conf >= 0.9) return 'high'
  if (conf >= 0.7) return 'mid'
  return 'low'
}

const effectiveBase64 = computed(() => {
  const raw = pastedBase64.value.trim()
  if (!raw) return null
  return raw.includes(',') ? raw.split(',')[1] : raw
})

// ── Image load → capture natural dimensions for viewBox ──────────────────────
const onImageLoad = () => {
  if (!overlayImg.value) return
  imgNaturalW.value = overlayImg.value.naturalWidth
  imgNaturalH.value = overlayImg.value.naturalHeight
}

// ── File handling ─────────────────────────────────────────────────────────────
const triggerFileInput = () => fileInput.value?.click()

const handleFileSelect = (event) => {
  const file = event.target.files?.[0]
  if (file) processFile(file)
}

const handleDrop = (event) => {
  isDragging.value = false
  const file = event.dataTransfer.files?.[0]
  if (file) processFile(file)
}

const processFile = (file) => {
  if (!file.type.startsWith('image/')) {
    error.value = 'Please select an image file (JPG, PNG, or WebP).'
    return
  }
  selectedFile.value = file
  const reader = new FileReader()
  reader.onload = (e) => {
    imagePreview.value = e.target.result
    pastedBase64.value = e.target.result.split(',')[1]
  }
  reader.readAsDataURL(file)
}

const clearImage = () => {
  selectedFile.value   = null
  imagePreview.value   = null
  pastedBase64.value   = ''
  preview.value        = null
  imgNaturalW.value    = 0
  imgNaturalH.value    = 0
  hoveredCandidate.value = null
  if (fileInput.value) fileInput.value.value = ''
}

const onBase64Paste = () => {
  const raw = pastedBase64.value.trim()
  if (raw.startsWith('data:image/')) imagePreview.value = raw
}

// ── Tauri calls ───────────────────────────────────────────────────────────────
const loadShops = async () => {
  try {
    shops.value = await invoke('load_shops')
  } catch (e) {
    console.error('load_shops failed:', e)
  }
}

const scanCoupon = async () => {
  scanning.value = true
  error.value    = null
  // Reset overlay dimensions so they recompute when the img remounts
  imgNaturalW.value = 0
  imgNaturalH.value = 0
  try {
    const result = await invoke('scan_coupon_image', {
      request: {
        image_base64: effectiveBase64.value,
        image_path:   selectedFile.value?.path ?? null,
      },
    })
    preview.value          = result
    selectedCandidate.value = result.best_index ?? 0
    await loadShops()
  } catch (e) {
    error.value = typeof e === 'string' ? e : 'Failed to scan image.'
    console.error(e)
  } finally {
    scanning.value = false
  }
}

const saveCoupon = async () => {
  if (!selectedShopId.value) { error.value = 'Please select a shop.'; return }
  if (!preview.value?.candidates?.[selectedCandidate.value]) {
    error.value = 'Please select a valid code.'; return
  }
  saving.value = true
  error.value  = null
  try {
    await invoke('save_coupon', {
      request: {
        candidates:               preview.value.candidates,
        selected_candidate_index: selectedCandidate.value,
        description:              description.value,
        shop_id:                  selectedShopId.value,
      },
    })
    clearImage()
    description.value    = ''
    selectedShopId.value = ''
    emit('saved')
  } catch (e) {
    error.value = typeof e === 'string' ? e : 'Failed to save coupon.'
    console.error(e)
  } finally {
    saving.value = false
  }
}

onMounted(loadShops)
</script>

<style scoped>
/* ── Layout ── */
.add-view {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

/* ── Upload card ── */
.upload-card {
  background: var(--surface, #fff);
  border: 1px solid var(--border, #e5e7eb);
  border-radius: 14px;
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.drop-zone {
  border: 2px dashed var(--border, #d1d5db);
  border-radius: 10px;
  min-height: 220px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: border-color 0.2s, background 0.2s;
  overflow: hidden;
  position: relative;
}

/* When preview is active the zone just wraps the image, no more dashed border */
.drop-zone.compact {
  border-style: solid;
  border-color: var(--border, #e5e7eb);
  min-height: unset;
  cursor: default;
}

.drop-zone:not(.compact):hover,
.drop-zone.drag-over {
  border-color: var(--accent, #6366f1);
  background: color-mix(in srgb, var(--accent, #6366f1) 5%, transparent);
}

.drop-placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  color: var(--text-muted, #9ca3af);
  text-align: center;
  padding: 24px;
  cursor: pointer;
}

.upload-icon {
  width: 64px;
  height: 64px;
  border-radius: 50%;
  background: color-mix(in srgb, var(--accent, #6366f1) 10%, transparent);
  color: var(--accent, #6366f1);
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 4px;
}

.drop-label  { margin: 0; font-weight: 600; font-size: 15px; color: var(--text, #111827); }
.drop-hint   { font-size: 12px; }

/* ── Preview wrapper ── */
.preview-wrapper {
  width: 100%;
  position: relative;
}

.preview-img {
  display: block;
  width: 100%;
  max-height: 400px;
  object-fit: contain;
  border-radius: 8px;
}

/* ── Overlay (image + SVG) ── */
.image-overlay-root {
  position: relative;
  display: inline-block; /* shrink-wrap to image */
  width: 100%;
  line-height: 0;
}

.overlay-img {
  display: block;
  width: 100%;
  max-height: 500px;
  object-fit: contain;
  border-radius: 8px;
  user-select: none;
}

.overlay-svg {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  pointer-events: none; /* let g elements handle events */
  border-radius: 8px;
}

/* Re-enable pointer on individual regions */
.overlay-svg .code-region {
  pointer-events: all;
}

/* ── Clear button ── */
.clear-btn {
  position: absolute;
  top: 8px;
  right: 8px;
  width: 28px;
  height: 28px;
  border-radius: 50%;
  background: rgba(0,0,0,0.55);
  color: #fff;
  border: none;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.2s;
  z-index: 10;
}

.clear-btn:hover { background: rgba(0,0,0,0.8); }

/* ── Alt input ── */
.alt-input {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.alt-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-muted, #9ca3af);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.alt-input textarea {
  border: 1px solid var(--border, #e5e7eb);
  border-radius: 8px;
  padding: 8px 10px;
  font-size: 12px;
  font-family: 'Courier New', monospace;
  color: var(--text, #111827);
  background: var(--bg, #f9fafb);
  resize: vertical;
  transition: border-color 0.2s;
}

.alt-input textarea:focus {
  outline: none;
  border-color: var(--accent, #6366f1);
}

/* ── Scan button ── */
.scan-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 11px 20px;
  background: var(--accent, #6366f1);
  color: #fff;
  border: none;
  border-radius: 9px;
  font-weight: 600;
  font-size: 14px;
  cursor: pointer;
  transition: opacity 0.2s, transform 0.15s;
}

.scan-btn:hover:not(:disabled) { opacity: 0.88; transform: translateY(-1px); }
.scan-btn:disabled { opacity: 0.5; cursor: not-allowed; }

/* ── Error banner ── */
.error-banner {
  display: flex;
  align-items: center;
  gap: 10px;
  background: #fef2f2;
  border: 1px solid #fecaca;
  border-radius: 10px;
  padding: 12px 14px;
  color: #dc2626;
  font-size: 14px;
}

.dismiss-btn {
  margin-left: auto;
  background: none;
  border: none;
  cursor: pointer;
  color: #dc2626;
  font-size: 14px;
}

/* ── Results section ── */
.results-section {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.section-header {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}

.section-header h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 700;
  color: var(--text, #111827);
}

.badge {
  background: color-mix(in srgb, var(--accent, #6366f1) 12%, transparent);
  color: var(--accent, #6366f1);
  padding: 2px 10px;
  border-radius: 20px;
  font-size: 12px;
  font-weight: 600;
}

.hint-text {
  font-size: 12px;
  color: var(--text-muted, #9ca3af);
  font-style: italic;
}

/* ── Candidate cards ── */
.candidates-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
  gap: 10px;
}

.candidate-card {
  background: var(--surface, #fff);
  border: 2px solid var(--border, #e5e7eb);
  border-radius: 12px;
  padding: 12px 14px 12px 0;
  cursor: pointer;
  display: flex;
  flex-direction: column;
  gap: 8px;
  position: relative;
  overflow: hidden;
  transition: border-color 0.15s, box-shadow 0.15s;
}

/* Left colour swatch strip */
.card-swatch {
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 5px;
  border-radius: 12px 0 0 12px;
  transition: width 0.15s;
}

.candidate-card:hover .card-swatch,
.candidate-card.hovered .card-swatch { width: 8px; }

.candidate-card:hover,
.candidate-card.hovered { border-color: var(--border, #c7c7d0); }

.candidate-card.selected {
  border-color: var(--accent, #6366f1);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent, #6366f1) 15%, transparent);
}

.candidate-meta {
  display: flex;
  align-items: center;
  gap: 6px;
  padding-left: 16px;
}

.index-badge {
  color: #fff;
  padding: 1px 7px;
  border-radius: 20px;
  font-size: 11px;
  font-weight: 800;
  font-family: monospace;
}

.type-pill {
  background: var(--bg, #f3f4f6);
  color: var(--text-secondary, #374151);
  padding: 2px 8px;
  border-radius: 20px;
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
}

.conf-pill {
  padding: 2px 8px;
  border-radius: 20px;
  font-size: 11px;
  font-weight: 600;
}

.conf-pill.high { background: #dcfce7; color: #16a34a; }
.conf-pill.mid  { background: #fef9c3; color: #ca8a04; }
.conf-pill.low  { background: #fee2e2; color: #dc2626; }

.candidate-value {
  font-family: 'Courier New', monospace;
  font-size: 14px;
  font-weight: 700;
  color: var(--text, #111827);
  background: var(--bg, #f9fafb);
  padding: 7px 10px 7px 16px;
  border-radius: 6px;
  margin: 0 10px 0 0;
  letter-spacing: 0.4px;
  word-break: break-all;
}

.check-mark {
  display: flex;
  align-items: center;
  gap: 5px;
  padding-left: 16px;
  font-size: 12px;
  font-weight: 700;
  color: var(--accent, #6366f1);
}

/* ── Assign card ── */
.assign-card {
  background: var(--surface, #fff);
  border: 1px solid var(--border, #e5e7eb);
  border-radius: 14px;
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.assign-title {
  margin: 0;
  font-size: 16px;
  font-weight: 700;
  color: var(--text, #111827);
}

.field-row {
  display: grid;
  grid-template-columns: 2fr 1fr;
  gap: 14px;
}

.field { display: flex; flex-direction: column; gap: 6px; }

.field label {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-secondary, #374151);
}

.optional { font-weight: 400; color: var(--text-muted, #9ca3af); }
.required { color: #dc2626; }

.text-input {
  border: 1px solid var(--border, #e5e7eb);
  border-radius: 8px;
  padding: 9px 12px;
  font-size: 14px;
  color: var(--text, #111827);
  background: var(--bg, #fff);
  transition: border-color 0.2s, box-shadow 0.2s;
}

.text-input:focus {
  outline: none;
  border-color: var(--accent, #6366f1);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent, #6366f1) 12%, transparent);
}

.select-wrapper { position: relative; }

.select-input {
  width: 100%;
  appearance: none;
  border: 1px solid var(--border, #e5e7eb);
  border-radius: 8px;
  padding: 9px 32px 9px 12px;
  font-size: 14px;
  color: var(--text, #111827);
  background: var(--bg, #fff);
  cursor: pointer;
  transition: border-color 0.2s;
}

.select-input:focus {
  outline: none;
  border-color: var(--accent, #6366f1);
}

.select-arrow {
  position: absolute;
  right: 10px;
  top: 50%;
  transform: translateY(-50%);
  pointer-events: none;
  color: var(--text-muted, #9ca3af);
}

.save-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 12px 24px;
  background: #16a34a;
  color: #fff;
  border: none;
  border-radius: 9px;
  font-weight: 600;
  font-size: 14px;
  cursor: pointer;
  transition: opacity 0.2s, transform 0.15s;
}

.save-btn:hover:not(:disabled) { opacity: 0.88; transform: translateY(-1px); }
.save-btn:disabled { opacity: 0.5; cursor: not-allowed; }

/* ── Spinner ── */
.btn-spinner {
  display: inline-block;
  width: 16px;
  height: 16px;
  border: 2px solid rgba(255,255,255,0.4);
  border-top-color: #fff;
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}

@keyframes spin { to { transform: rotate(360deg); } }

/* ── Transitions ── */
.fade-enter-active, .fade-leave-active { transition: opacity 0.2s; }
.fade-enter-from,  .fade-leave-to      { opacity: 0; }

.slide-down-enter-active { transition: all 0.25s ease; }
.slide-down-enter-from   { opacity: 0; transform: translateY(-8px); }

.slide-up-enter-active { transition: all 0.3s ease; }
.slide-up-enter-from   { opacity: 0; transform: translateY(12px); }

/* ── Responsive ── */
@media (max-width: 600px) {
  .field-row       { grid-template-columns: 1fr; }
  .candidates-grid { grid-template-columns: 1fr; }
}
</style>