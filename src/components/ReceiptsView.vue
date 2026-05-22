<template>
  <ReceiptView
    v-if="detailReceiptId"
    :receipt-id="detailReceiptId"
    @back="detailReceiptId = null"
    @go-to-shop="$emit('go-to-shop', $event)"
  />

  <div v-else>
    <div class="card">
      <h2>Scan Receipt</h2>
      <div class="scan-section">
        <!-- Kamera lub wybór pliku -->
        <input 
          type="file" 
          accept="image/*" 
          capture="environment" 
          @change="handleImageUpload" 
        />
        
        <div v-if="imagePreviewUrl" class="image-tools">
          <button type="button" @click="setZoom(imageZoom - 0.15)">-</button>
          <input v-model.number="imageZoom" type="range" min="0.25" max="3" step="0.05" />
          <button type="button" @click="setZoom(imageZoom + 0.15)">+</button>
          <button type="button" @click="setZoom(1)">100%</button>
          <span>{{ Math.round(imageZoom * 100) }}%</span>
        </div>

        <div v-if="imagePreviewUrl" class="image-viewport">
          <div class="image-preview" ref="imageStage">
            <img :src="imagePreviewUrl" alt="Receipt Preview" :style="imageStyle" @load="measureImage" />
            <button
              v-for="field in preview?.field_candidates ?? []"
              :key="field.field_id"
              class="ocr-box"
              :class="[field.role, { selected: field.selected }]"
              :style="boxStyle(field.bounding_box)"
              type="button"
              :title="`${field.label}: ${field.value_text}`"
              @click="field.selected = !field.selected"
            />
          </div>
        </div>

        <textarea
          v-model="ocrBlocksJson"
          spellcheck="false"
          placeholder='Optional OCR box JSON override: [{"text":"Coffee","bounding_box":{"x":40,"y":120,"width":90,"height":20}}]'
        />

        <button @click="scanReceipt" :disabled="scanning || (!imageBase64 && !ocrBlocksJson.trim())" style="margin-top: 10px;">
          {{ scanning ? 'Scanning...' : 'Scan Receipt' }}
        </button>
      </div>

      <div v-if="preview" class="preview">
        <h3>Receipt Preview</h3>
        <div class="form-group">
          <label>Shop:</label>
          <select v-model="selectedShopId">
            <option value="">Select existing shop or create new</option>
            <option v-for="shop in shops" :key="shop.shop_id" :value="shop.shop_id">
              {{ shop.shop_name }}
            </option>
          </select>
          <input 
            v-if="!selectedShopId" 
            v-model="newShopName" 
            placeholder="Or enter new shop name" 
          />
        </div>
        
        <div class="receipt-data">
          <p><strong>Raw Shop Name:</strong> {{ preview.raw_shop_name }}</p>
          <label>Total</label>
          <input v-model.number="preview.total_value" type="number" step="0.01" />
          <label>Discount</label>
          <input v-model.number="preview.total_discount" type="number" step="0.01" />

          <h4>Detected fields:</h4>
          <div class="field-grid">
            <label v-for="field in preview.field_candidates" :key="field.field_id" class="field-candidate">
              <input v-model="field.selected" type="checkbox" />
              <span>{{ field.label }}</span>
              <strong>{{ field.value_text }}</strong>
            </label>
          </div>
          
          <h4>Items:</h4>
          <div v-for="(item, idx) in preview.entries" :key="idx" class="receipt-item">
            <input v-model="item.entry_name" class="item-name" />
            <input v-model.number="item.entry_quantity" class="item-qty" type="number" min="1" />
            <input v-model.number="item.entry_cost" class="item-cost" type="number" step="0.01" />
            <input v-model.number="item.entry_discount" class="item-discount" type="number" step="0.01" />
            <button type="button" @click="preview.entries.splice(idx, 1)">Remove</button>
          </div>
          <button type="button" @click="addReceiptEntry">Add item</button>
        </div>

        <button @click="saveReceipt" :disabled="saving">Save Receipt</button>
      </div>
    </div>

    <div class="card">
      <h2>Receipts List</h2>
      <div class="pagination">
        <button @click="loadReceipts(0)" :disabled="loading">Refresh</button>
        <button @click="loadReceipts(offset - limit)" :disabled="offset === 0">Previous</button>
        <button @click="loadReceipts(offset + limit)" :disabled="!hasMore">Next</button>
      </div>
      
      <div v-if="loading" class="loading">Loading...</div>
      <div v-else-if="error" class="error">{{ error }}</div>
      <div v-else>
        <div v-for="receipt in receipts" :key="receipt.receipt_id" class="receipt-item-summary">
          <div class="receipt-header" @click="openReceipt(receipt.receipt_id)">
            <div class="receipt-shop-logo" @click.stop="$emit('go-to-shop', receipt.shop_id)">
              <img v-if="shopLogo(receipt.shop_id)" :src="`data:image/png;base64,${shopLogo(receipt.shop_id)}`" :alt="receipt.shop_name" />
              <span v-else>{{ receipt.shop_name.charAt(0).toUpperCase() }}</span>
            </div>
            <div class="receipt-summary-text">
              <strong>{{ receipt.shop_name }}</strong>
              <small>{{ receipt.total_discount > 0 ? `Discount $${receipt.total_discount.toFixed(2)}` : 'Receipt' }}</small>
            </div>
            <span class="total">${{ receipt.total_value.toFixed(2) }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import ReceiptView from './ReceiptView.vue'

const props = defineProps({
  selectedReceiptId: { type: String, default: null },
})

defineEmits(['go-to-shop'])

const shops = ref([])
const receipts = ref([])
const loading = ref(false)
const error = ref(null)
const offset = ref(0)
const limit = ref(10)
const hasMore = ref(false)

// Scan state
const imageBase64 = ref('')
const imagePreviewUrl = ref('')
const imageStage = ref(null)
const imageNaturalSize = ref({ width: 1, height: 1 })
const imageZoom = ref(0.6)
const ocrBlocksJson = ref('')
const scanning = ref(false)
const saving = ref(false)
const preview = ref(null)
const selectedShopId = ref('')
const newShopName = ref('')

const detailReceiptId = ref(null)

// File upload handler (Kamera na smartfonie / plik na desktopie)
const handleImageUpload = (event) => {
  const file = event.target.files[0];
  if (!file) return;

  // Stwórzmy z pliku URL dla podglądu dla użytkownika
  imagePreviewUrl.value = URL.createObjectURL(file);

  // Przekonwertowanie pliku na Base64 dla Rusta
  const reader = new FileReader();
  reader.onload = () => {
    const dataUrl = reader.result;
    // Otrzymany string ma postać: "data:image/jpeg;base64,/9j/4AAQSkZJRg..."
    // Backend w RUST (base64::engine::general_purpose::STANDARD.decode) wymaga czystego base64.
    // Musimy uciąć prefix.
    const base64String = dataUrl.split(',')[1];
    imageBase64.value = base64String;
  };
  reader.readAsDataURL(file);
};
const measureImage = (event) => {
  const image = event.target
  imageNaturalSize.value = {
    width: image.naturalWidth || 1,
    height: image.naturalHeight || 1
  }
}

const imageStyle = computed(() => ({
  width: `${Math.max(imageNaturalSize.value.width * imageZoom.value, 1)}px`
}))

const setZoom = (value) => {
  imageZoom.value = Math.min(3, Math.max(0.25, Number(value.toFixed(2))))
}

const boxStyle = (box) => {
  const scaleX = imageZoom.value
  const scaleY = imageZoom.value

  return {
    left: `${box.x * scaleX}px`,
    top: `${box.y * scaleY}px`,
    width: `${Math.max(box.width * scaleX, 8)}px`,
    height: `${Math.max(box.height * scaleY, 8)}px`
  }
}

const loadShops = async () => {
  try {
    shops.value = await invoke('load_shops')
  } catch (err) {
    console.error(err)
  }
}

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

const scanReceipt = async () => {
  if (!imageBase64.value && !ocrBlocksJson.value.trim()) return;

  scanning.value = true;
  error.value = null;
  try {
    const result = ocrBlocksJson.value.trim()
      ? await analyzeOcrBlocks()
      : await invoke('scan_receipt_image', {
          request: {
            image_base64: imageBase64.value
          }
        });

    preview.value = result;
    selectedShopId.value = result.suggested_shop_id || '';
    newShopName.value = result.raw_shop_name || '';
    await loadShops();
  } catch (err) {
    error.value = String(err?.message ?? err);
    console.error("Receipt scan error:", err);
    alert("Receipt scan failed: " + error.value);
  } finally {
    scanning.value = false;
  }
}

const shopLogo = (shopId) => shops.value.find((shop) => shop.shop_id === shopId)?.logo_base64

const analyzeOcrBlocks = async () => {
  const blocksInput = JSON.parse(ocrBlocksJson.value)
  const blocks = Array.isArray(blocksInput) ? blocksInput : blocksInput.blocks

  if (!Array.isArray(blocks)) {
    throw new Error('OCR JSON must be an array or an object with a blocks array')
  }

  return await invoke('scan_receipt_ocr_blocks', {
    request: {
      blocks
    }
  });
}

const saveReceipt = async () => {
  saving.value = true
  error.value = null
  try {
    await invoke('save_receipt', {
      request: {
        shop_id: selectedShopId.value,
        new_shop_name: newShopName.value || null,
        total_value: preview.value.total_value,
        total_discount: preview.value.total_discount,
        entries: preview.value.entries
      }
    })
    await loadReceipts(0)
    // Reset form
    preview.value = null
    selectedShopId.value = ''
    newShopName.value = ''
    imageBase64.value = ''
    imagePreviewUrl.value = ''
  } catch (err) {
    error.value = err
    console.error(err)
  } finally {
    saving.value = false
  }
}

const addReceiptEntry = () => {
  if (!preview.value) return

  preview.value.entries.push({
    draft_id: crypto.randomUUID(),
    entry_name: '',
    entry_quantity: 1,
    entry_cost: 0,
    entry_discount: 0
  })
}

const openReceipt = (receiptId) => {
  detailReceiptId.value = receiptId
}

loadReceipts(0)
loadShops()

watch(
  () => props.selectedReceiptId,
  (receiptId) => {
    if (receiptId) openReceipt(receiptId)
  },
  { immediate: true }
)
</script>

<style scoped>
.scan-section {
  display: flex;
  flex-direction: column;
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

.image-tools {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.image-tools input {
  width: 180px;
}

.image-tools span {
  min-width: 44px;
  font-size: 13px;
  color: #555;
}

.image-viewport {
  max-width: 100%;
  max-height: 560px;
  overflow: auto;
  border: 1px solid #e5e5e5;
  border-radius: 8px;
  background: #f7f7f7;
}

.image-preview {
  position: relative;
  width: fit-content;
  min-width: 1px;
  min-height: 1px;
}

.image-preview img {
  display: block;
  border-radius: 8px;
}

.ocr-box {
  position: absolute;
  padding: 0;
  border: 2px dashed #667eea;
  background: rgba(102, 126, 234, 0.05);
  border-radius: 3px;
  cursor: pointer;
}

.ocr-box.selected {
  border-style: solid;
  background: rgba(102, 126, 234, 0.16);
}

.ocr-box.total {
  border-color: #059669;
}

.ocr-box.total.selected {
  background: rgba(5, 150, 105, 0.16);
}

.ocr-box.total_discount,
.ocr-box.item_price {
  border-color: #d97706;
}

.ocr-box.total_discount.selected,
.ocr-box.item_price.selected {
  background: rgba(217, 119, 6, 0.16);
}

.receipt-data {
  background: #f9f9f9;
  padding: 15px;
  border-radius: 8px;
  margin: 15px 0;
}

.receipt-item {
  display: flex;
  gap: 15px;
  padding: 8px;
  border-bottom: 1px solid #eee;
  font-size: 14px;
  align-items: center;
}

.item-name {
  flex: 2;
  font-weight: 500;
}

.item-qty, .item-cost, .item-discount {
  flex: 1;
}

.field-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: 8px;
  margin: 10px 0 15px;
}

.field-candidate {
  display: grid;
  grid-template-columns: auto 1fr;
  gap: 4px 8px;
  padding: 8px;
  border: 1px solid #eee;
  border-radius: 6px;
}

.field-candidate input {
  width: auto;
  grid-row: span 2;
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
  align-items: center;
  gap: 12px;
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

.receipt-shop-logo {
  width: 52px;
  height: 52px;
  border-radius: 8px;
  overflow: hidden;
  background: #eef2ff;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.receipt-shop-logo img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.receipt-shop-logo span {
  color: #667eea;
  font-weight: 800;
  font-size: 20px;
}

.receipt-summary-text {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 3px;
  min-width: 0;
}

.receipt-summary-text small {
  color: #777;
}

.pagination {
  display: flex;
  gap: 10px;
  align-items: center;
  margin-bottom: 20px;
}
</style>
