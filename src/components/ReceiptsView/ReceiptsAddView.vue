<template>
  <div class="add-view">
    <div class="card">
      <h2>Add Receipt</h2>

      <div class="scan-section">
        <input
          ref="receiptFileInput"
          type="file"
          accept="image/*"
          capture="environment"
          class="receipt-file-input"
          @change="handleImageUpload"
        />

        <button class="scan-trigger" @click="triggerFileInput">
          Take photo / choose file
        </button>

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
  </div>
</template>

<script setup>
import { computed, nextTick, ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const emit = defineEmits(['saved'])

const shops = ref([])
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
const receiptFileInput = ref(null)

const measureImage = (event) => {
  const image = event.target
  imageNaturalSize.value = {
    width: image.naturalWidth || 1,
    height: image.naturalHeight || 1,
  }
}

const imageStyle = computed(() => ({
  width: `${Math.max(imageNaturalSize.value.width * imageZoom.value, 1)}px`,
}))

const setZoom = (value) => {
  imageZoom.value = Math.min(3, Math.max(0.25, Number(value.toFixed(2))))
}

const triggerFileInput = () => {
  receiptFileInput.value?.click()
}

const boxStyle = (box) => {
  const scaleX = imageZoom.value
  const scaleY = imageZoom.value
  return {
    left: `${box.x * scaleX}px`,
    top: `${box.y * scaleY}px`,
    width: `${Math.max(box.width * scaleX, 8)}px`,
    height: `${Math.max(box.height * scaleY, 8)}px`,
  }
}

const handleImageUpload = (event) => {
  const file = event.target.files[0]
  if (!file) return

  imagePreviewUrl.value = URL.createObjectURL(file)

  const reader = new FileReader()
  reader.onload = () => {
    const dataUrl = reader.result
    imageBase64.value = dataUrl.split(',')[1]
  }
  reader.readAsDataURL(file)
}

const loadShops = async () => {
  try {
    shops.value = await invoke('load_shops')
  } catch (err) {
    console.error(err)
  }
}

const scanReceipt = async () => {
  if (!imageBase64.value && !ocrBlocksJson.value.trim()) return

  scanning.value = true
  try {
    const result = ocrBlocksJson.value.trim()
      ? await analyzeOcrBlocks()
      : await invoke('scan_receipt_image', {
          request: { image_base64: imageBase64.value },
        })

    preview.value = result
    selectedShopId.value = result.suggested_shop_id || ''
    newShopName.value = result.raw_shop_name || ''
    await loadShops()
  } catch (err) {
    console.error(err)
  } finally {
    scanning.value = false
  }
}

const analyzeOcrBlocks = async () => {
  const blocksInput = JSON.parse(ocrBlocksJson.value)
  const blocks = Array.isArray(blocksInput) ? blocksInput : blocksInput.blocks

  if (!Array.isArray(blocks)) {
    throw new Error('OCR JSON must be an array or an object with a blocks array')
  }

  return await invoke('scan_receipt_ocr_blocks', { request: { blocks } })
}

const saveReceipt = async () => {
  saving.value = true
  try {
    await invoke('save_receipt', {
      request: {
        shop_id: selectedShopId.value,
        new_shop_name: newShopName.value || null,
        total_value: preview.value.total_value,
        total_discount: preview.value.total_discount,
        entries: preview.value.entries,
      },
    })

    preview.value = null
    selectedShopId.value = ''
    newShopName.value = ''
    imageBase64.value = ''
    imagePreviewUrl.value = ''
    ocrBlocksJson.value = ''
    emit('saved')
  } catch (err) {
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
    entry_discount: 0,
  })
}

onMounted(async () => {
  await loadShops()
  await nextTick()
  triggerFileInput()
})
</script>

<style scoped>
.add-view {
  display: flex;
  flex-direction: column;
}

.scan-section {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin-bottom: 20px;
}

.receipt-file-input {
  display: none;
}

.scan-trigger {
  align-self: flex-start;
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

.item-qty,
.item-cost,
.item-discount {
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
</style>
