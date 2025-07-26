<template>
  <div class="torrent-search">
    <div class="search-container">
      <h1>Torrent Search</h1>
      
      <div class="search-form">
        <div class="search-input-group">
          <input
            v-model="searchKeyword"
            type="text"
            placeholder="Enter search keyword..."
            class="search-input"
            @keyup.enter="searchTorrents"
            :disabled="loading"
          />
          <button 
            @click="searchTorrents" 
            class="search-button"
            :disabled="loading || !searchKeyword.trim()"
          >
            <span v-if="loading">Searching...</span>
            <span v-else>Search</span>
          </button>
        </div>
      </div>

      <div v-if="error" class="error-message">
        {{ error }}
      </div>

      <div v-if="downloadMessage" class="download-message">
        {{ downloadMessage }}
      </div>

      <div v-if="loading" class="loading">
        <div class="loading-spinner"></div>
        <p>Searching torrents...</p>
      </div>

      <div v-if="results.length > 0" class="results">
        <h2>Search Results ({{ results.length }} found)</h2>
        <div class="torrent-grid">
          <div v-for="torrent in results" :key="torrent.id" class="torrent-card">
            <div class="torrent-header">
              <h3 class="torrent-name">{{ torrent.name }}</h3>
              <div class="torrent-meta">
                <span class="torrent-size">{{ formatSize(torrent.size) }}</span>
                <span class="torrent-date">{{ formatDate(torrent.added) }}</span>
              </div>
            </div>
            
            <p class="torrent-description">{{ torrent.small_descr }}</p>
            
            <div class="torrent-stats">
              <div class="stat-group">
                <span class="stat-label">Seeders:</span>
                <span class="stat-value seeders">{{ torrent.seeders }}</span>
              </div>
              <div class="stat-group">
                <span class="stat-label">Leechers:</span>
                <span class="stat-value leechers">{{ torrent.leechers }}</span>
              </div>
              <div class="stat-group">
                <span class="stat-label">Completed:</span>
                <span class="stat-value">{{ torrent.times_completed }}</span>
              </div>
              <div class="stat-group">
                <span class="stat-label">H&R:</span>
                <span class="stat-value">{{ torrent.hr }}</span>
              </div>
            </div>

            <div v-if="torrent.promotion_time_type > 0" class="promotion-info">
              <span class="promotion-badge">
                Promotion until {{ formatDate(torrent.promotion_until) }}
              </span>
            </div>

            <div class="torrent-actions">
              <button class="action-button download-btn" @click="downloadTorrent(torrent)" :disabled="downloading === torrent.id">
                <span v-if="downloading === torrent.id">Downloading...</span>
                <span v-else>📥 Download</span>
              </button>
              <button class="action-button info-hash" @click="copyToClipboard(torrent.info_hash)">
                Copy Info Hash
              </button>
              <button class="action-button download" @click="copyToClipboard(torrent.downhash)">
                Copy Download Hash
              </button>
            </div>
          </div>
        </div>
      </div>

      <div v-if="searched && results.length === 0 && !loading && !error" class="no-results">
        <h2>No torrents found</h2>
        <p>Try different keywords or check your spelling.</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useRoute } from 'vue-router'
import { API_CONFIG } from '@/config/api'

interface Torrent {
  id: number
  promotion_time_type: number
  promotion_until: string
  leechers: number
  seeders: number
  name: string
  small_descr: string
  times_completed: number
  size: number
  added: string
  hr: number
  info_hash: string
  downhash: string
}

interface SearchResponse {
  status: number
  data: Torrent[]
}

const route = useRoute()

const searchKeyword = ref('')
const results = ref<Torrent[]>([])
const loading = ref(false)
const error = ref('')
const searched = ref(false)
const downloading = ref<number | null>(null)
const downloadMessage = ref('')

// Helper function to build API URLs with token
const buildApiUrl = (endpoint: string, params: Record<string, string> = {}) => {
  const token = route.query.token as string
  const url = new URL(endpoint, API_CONFIG.BASE_URL)
  
  // Add token if present
  if (token) {
    url.searchParams.set('token', token)
  }
  
  // Add other parameters
  Object.entries(params).forEach(([key, value]) => {
    url.searchParams.set(key, value)
  })
  
  return url.toString()
}

const searchTorrents = async () => {
  if (!searchKeyword.value.trim()) return
  
  loading.value = true
  error.value = ''
  results.value = []
  
  try {
    const apiUrl = buildApiUrl('/search', { keyword: searchKeyword.value.trim() })
    const response = await fetch(apiUrl)
    
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`)
    }
    
    const data: SearchResponse = await response.json()
    
    if (data.status === 0) {
      results.value = data.data
      searched.value = true
    } else {
      error.value = 'Search failed. Please try again.'
    }
  } catch (err) {
    error.value = 'Failed to connect to search API. Make sure the backend is running on localhost:3000.'
    console.error('Search error:', err)
  } finally {
    loading.value = false
  }
}

const formatSize = (bytes: number): string => {
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  let size = bytes
  let unitIndex = 0
  
  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024
    unitIndex++
  }
  
  return `${size.toFixed(2)} ${units[unitIndex]}`
}

const formatDate = (dateString: string): string => {
  const date = new Date(dateString)
  return date.toLocaleDateString() + ' ' + date.toLocaleTimeString()
}

const downloadTorrent = async (torrent: Torrent) => {
  downloading.value = torrent.id
  downloadMessage.value = ''
  
  try {
    const apiUrl = buildApiUrl('/download', { 
      id: torrent.id.toString(), 
      downhash: torrent.downhash 
    })
    const response = await fetch(apiUrl)
    
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`)
    }
    
    const data = await response.json()
    
    if (data.status === 'success') {
      downloadMessage.value = data.message || 'Download started successfully!'
      // Clear the success message after 3 seconds
      setTimeout(() => {
        downloadMessage.value = ''
      }, 3000)
    } else {
      downloadMessage.value = 'Download failed. Please try again.'
    }
  } catch (err) {
    downloadMessage.value = 'Failed to start download. Make sure the backend is running.'
    console.error('Download error:', err)
  } finally {
    downloading.value = null
  }
}

const copyToClipboard = async (text: string) => {
  try {
    await navigator.clipboard.writeText(text)
    // You could add a toast notification here
    console.log('Copied to clipboard:', text)
  } catch (err) {
    console.error('Failed to copy to clipboard:', err)
  }
}
</script>

<style scoped>
.torrent-search {
  width: 100%;
  margin: 0 auto;
  padding: 2rem;
}

.search-container {
  width: 100%;
}

h1 {
  text-align: center;
  color: var(--color-heading);
  margin-bottom: 2rem;
  font-size: 2.5rem;
}

.search-form {
  margin-bottom: 2rem;
}

.search-input-group {
  display: flex;
  gap: 1rem;
  width: 100%;
  max-width: 1000px;
  margin: 0 auto;
}

.search-input {
  flex: 1;
  padding: 0.75rem 1rem;
  border: 2px solid var(--color-border);
  border-radius: 8px;
  font-size: 1rem;
  transition: border-color 0.3s;
}

.search-input:focus {
  outline: none;
  border-color: var(--color-border-hover);
}

.search-button {
  padding: 0.75rem 1.5rem;
  background-color: var(--color-background-mute);
  color: var(--color-text);
  border: 2px solid var(--color-border);
  border-radius: 8px;
  font-size: 1rem;
  cursor: pointer;
  transition: all 0.3s;
  min-width: 120px;
}

.search-button:hover:not(:disabled) {
  background-color: var(--color-background-soft);
  border-color: var(--color-border-hover);
}

.search-button:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.error-message {
  background-color: #fee;
  color: #c33;
  padding: 1rem;
  border-radius: 8px;
  margin-bottom: 2rem;
  text-align: center;
}

.loading {
  text-align: center;
  padding: 2rem;
}

.loading-spinner {
  width: 40px;
  height: 40px;
  border: 4px solid var(--color-border);
  border-top: 4px solid var(--color-text);
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin: 0 auto 1rem;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

.results h2 {
  color: var(--color-heading);
  margin-bottom: 1.5rem;
}

.torrent-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(400px, 1fr));
  gap: 1.5rem;
}

.torrent-card {
  background: var(--color-background-mute);
  border: 1px solid var(--color-border);
  border-radius: 12px;
  padding: 1.5rem;
  transition: all 0.3s;
}

.torrent-card:hover {
  border-color: var(--color-border-hover);
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.torrent-header {
  margin-bottom: 1rem;
}

.torrent-name {
  color: var(--color-heading);
  font-size: 1.1rem;
  margin-bottom: 0.5rem;
  line-height: 1.4;
}

.torrent-meta {
  display: flex;
  gap: 1rem;
  font-size: 0.9rem;
  color: var(--color-text-light);
}

.torrent-description {
  color: var(--color-text);
  margin-bottom: 1rem;
  font-style: italic;
}

.torrent-stats {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 0.5rem;
  margin-bottom: 1rem;
  font-size: 0.9rem;
}

.stat-group {
  display: flex;
  justify-content: space-between;
}

.stat-label {
  color: var(--color-text-light);
}

.stat-value {
  font-weight: 600;
}

.stat-value.seeders {
  color: #059669;
}

.stat-value.leechers {
  color: #dc2626;
}

.promotion-info {
  margin-bottom: 1rem;
}

.promotion-badge {
  background: linear-gradient(45deg, #ffd700, #ffed4e);
  color: #1f2937;
  padding: 0.25rem 0.75rem;
  border-radius: 20px;
  font-size: 0.8rem;
  font-weight: 600;
}

.torrent-actions {
  display: flex;
  gap: 0.5rem;
  flex-wrap: wrap;
}

.action-button {
  padding: 0.5rem 1rem;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  background: var(--color-background);
  color: var(--color-text);
  cursor: pointer;
  font-size: 0.8rem;
  transition: all 0.3s;
}

.action-button:hover:not(:disabled) {
  background: var(--color-background-soft);
  border-color: var(--color-border-hover);
}

.action-button:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.download-btn {
  background: #10b981;
  color: white;
  border-color: #10b981;
}

.download-btn:hover:not(:disabled) {
  background: #059669;
  border-color: #059669;
}

.download-message {
  background-color: #dcfce7;
  color: #166534;
  padding: 1rem;
  border-radius: 8px;
  margin-bottom: 2rem;
  text-align: center;
  border: 1px solid #bbf7d0;
}

.no-results {
  text-align: center;
  padding: 3rem;
  color: var(--color-text-light);
}

.no-results h2 {
  color: var(--color-heading);
  margin-bottom: 1rem;
}

@media (max-width: 768px) {
  .torrent-search {
    padding: 1rem;
  }
  
  .search-input-group {
    flex-direction: column;
  }
  
  .torrent-grid {
    grid-template-columns: 1fr;
  }
  
  .torrent-stats {
    grid-template-columns: 1fr;
  }
}
</style> 