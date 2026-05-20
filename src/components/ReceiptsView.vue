<template>
  <div>
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
        
        <!-- Podgląd zrobionego zdjęcia przed wysłaniem na OCR -->
        <div v-if="imagePreviewUrl" class="image-preview">
          <img :src="imagePreviewUrl" alt="Receipt Preview" style="max-width: 100%; max-height: 300px; border-radius: 8px; margin-top: 10px;" />
        </div>

        <button @click="scanReceipt" :disabled="scanning || !imageBase64" style="margin-top: 10px;">
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
          <p><strong>Total:</strong> ${{ preview.total_value.toFixed(2) }}</p>
          <p><strong>Discount:</strong> ${{ preview.total_discount.toFixed(2) }}</p>
          
          <h4>Items:</h4>
          <div v-for="(item, idx) in preview.entries" :key="idx" class="receipt-item">
            <span class="item-name">{{ item.entry_name }}</span>
            <span class="item-qty">x{{ item.entry_quantity }}</span>
            <span class="item-cost">${{ item.entry_cost.toFixed(2) }}</span>
            <span v-if="item.entry_discount > 0" class="item-discount">-{{ item.entry_discount }}</span>
          </div>
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
          <div class="receipt-header" @click="viewDetail(receipt.receipt_id)">
            <strong>{{ receipt.shop_name }}</strong>
            <span class="total">${{ receipt.total_value.toFixed(2) }}</span>
          </div>
          <div class="receipt-detail" v-if="selectedReceipt === receipt.receipt_id">
            <div v-if="detailLoading">Loading details...</div>
            <div v-else-if="receiptDetail">
              <p><strong>Discount:</strong> ${{ receiptDetail.total_discount.toFixed(2) }}</p>
              <h5>Items:</h5>
              <div v-for="item in receiptDetail.entries" :key="item.entry_id" class="detail-item">
                {{ item.entry_name }} - {{ item.entry_quantity }} x ${{ item.entry_cost.toFixed(2) }}
                <span v-if="item.entry_discount > 0">(Discount: ${{ item.entry_discount }})</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

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
const scanning = ref(false)
const saving = ref(false)
const preview = ref(null)
const selectedShopId = ref('')
const newShopName = ref('')

// Detail view
const selectedReceipt = ref(null)
const receiptDetail = ref(null)

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
const detailLoading = ref(false)

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
  if (!imageBase64.value) return;

  scanning.value = true;
  error.value = null;
  try {
    // Zgodnie ze specyfikacją z agents.md lub Rusta będzie to wywoływało z backendu OCR.
    // Argument nosi nazwę image_base64, która w obiekcie trafia jako snake_case.
    const result = await invoke('create_receipt', {
      imageBase64: imageBase64.value
    });
    
    preview.value = result;
    await loadShops();
  } catch (err) {
    error.value = err;
    console.error("OCR Error:", err);
    alert("Wystąpił błąd OCR: " + err);
  } finally {
    scanning.value = false;
  }
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
    mockImageData.value = ''
  } catch (err) {
    error.value = err
    console.error(err)
  } finally {
    saving.value = false
  }
}

const viewDetail = async (receiptId) => {
  if (selectedReceipt.value === receiptId) {
    selectedReceipt.value = null
    receiptDetail.value = null
    return
  }
  
  selectedReceipt.value = receiptId
  detailLoading.value = true
  
  try {
    receiptDetail.value = await invoke('load_receipt_detail', { receiptId })
  } catch (err) {
    error.value = err
    console.error(err)
  } finally {
    detailLoading.value = false
  }
}

loadReceipts(0)
</script>

<style scoped>
.scan-section {
  display: flex;
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
}

.item-name {
  flex: 2;
  font-weight: 500;
}

.item-qty, .item-cost, .item-discount {
  flex: 1;
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

.receipt-detail {
  padding: 12px;
  border-top: 1px solid #eee;
  background: white;
}

.detail-item {
  padding: 5px 0;
  font-size: 14px;
  border-bottom: 1px solid #f0f0f0;
}

.pagination {
  display: flex;
  gap: 10px;
  align-items: center;
  margin-bottom: 20px;
}
</style>