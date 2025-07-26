<template>
  <div class="torrent-status">
    <div class="status-header">
      <h1>Torrent Status</h1>
      <button @click="refreshStatus" class="refresh-button" :disabled="loading">
        <span v-if="loading">🔄 Refreshing...</span>
        <span v-else>🔄 Refresh</span>
      </button>
    </div>

    <div v-if="error" class="error-message">
      {{ error }}
    </div>

    <div v-if="deleteMessage" class="delete-message">
      {{ deleteMessage }}
    </div>

    <div v-if="loading && torrents.length === 0" class="loading">
      <div class="loading-spinner"></div>
      <p>Loading torrent status...</p>
    </div>

    <div v-if="torrents.length === 0 && !loading && !error" class="no-torrents">
      <h2>No active torrents</h2>
      <p>No torrents are currently downloading or uploading.</p>
      <router-link :to="getRouteWithToken('/search')" class="search-link">Start searching for torrents</router-link>
    </div>

    <div v-if="torrents.length > 0" class="torrents-list">
      <div class="status-summary">
        <div class="summary-card">
          <h3>📥 Downloading</h3>
          <span class="count">{{ downloadingCount }}</span>
        </div>
        <div class="summary-card">
          <h3>✅ Completed</h3>
          <span class="count">{{ completedCount }}</span>
        </div>
        <div class="summary-card">
          <h3>❌ Error</h3>
          <span class="count">{{ errorCount }}</span>
        </div>
        <div class="summary-card">
          <h3>📊 Total</h3>
          <span class="count">{{ torrents.length }}</span>
        </div>
      </div>

      <div class="torrent-list">
        <div v-for="torrent in torrents" :key="torrent.hash" class="torrent-item">
                     <div class="torrent-header">
             <div class="torrent-info">
               <h3 class="torrent-name">{{ torrent.name }}</h3>
               <div class="torrent-meta">
                 <span class="torrent-size">{{ formatSize(torrent.size) }}</span>
                 <span class="torrent-path">{{ torrent.content_path }}</span>
               </div>
             </div>
             <div class="header-actions">
               <div class="status-badge" :class="getStatusClass(torrent.status)">
                 {{ getStatusText(torrent.status) }}
               </div>
               <button 
                 class="view-files-button" 
                 @click="viewFiles(torrent)" 
                 :disabled="loadingFiles === torrent.hash"
                 title="View and download files"
               >
                 <span v-if="loadingFiles === torrent.hash">⏳</span>
                 <span v-else>📁</span>
               </button>
               <button 
                 class="delete-button" 
                 @click="deleteTorrent(torrent)" 
                 :disabled="deleting === torrent.hash"
                 title="Stop and remove torrent"
               >
                 <span v-if="deleting === torrent.hash">⏳</span>
                 <span v-else>🗑️</span>
               </button>
             </div>
           </div>

          <div class="progress-section">
            <div class="progress-bar">
              <div 
                class="progress-fill" 
                :style="{ width: (torrent.progress * 100) + '%' }"
                :class="getProgressClass(torrent.status)"
              ></div>
            </div>
            <span class="progress-text">{{ (torrent.progress * 100).toFixed(1) }}%</span>
          </div>

          <div class="torrent-stats">
            <div class="stat-row">
              <div class="stat-item">
                <span class="stat-label">Download Speed:</span>
                <span class="stat-value download">{{ formatSpeed(torrent.dlspeed) }}</span>
              </div>
              <div class="stat-item">
                <span class="stat-label">Upload Speed:</span>
                <span class="stat-value upload">{{ formatSpeed(torrent.upspeed) }}</span>
              </div>
            </div>
            <div class="stat-row">
              <div class="stat-item">
                <span class="stat-label">ETA:</span>
                <span class="stat-value">{{ formatETA(torrent.eta, torrent.status) }}</span>
              </div>
              <div class="stat-item">
                <span class="stat-label">Hash:</span>
                <span class="stat-value hash" @click="copyToClipboard(torrent.hash)">
                  {{ torrent.hash.substring(0, 16) }}...
                </span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- File Viewer Modal -->
    <div v-if="showFileModal" class="modal-overlay" @click="closeFileModal">
      <div class="modal-content" @click.stop>
        <div class="modal-header">
          <h3>Files in: {{ selectedTorrent?.name }}</h3>
          <button class="close-button" @click="closeFileModal">✕</button>
        </div>
        
        <div class="modal-body">
          <div v-if="fileListError" class="error-message">
            {{ fileListError }}
          </div>
          
          <div v-if="loadingFileList" class="loading">
            <div class="loading-spinner"></div>
            <p>Loading files...</p>
          </div>
          
          <div v-if="fileList.length === 0 && !loadingFileList && !fileListError" class="no-files">
            <p>No files found in this torrent.</p>
          </div>
          
          <div v-if="fileList.length > 0" class="file-list">
            <div v-for="file in fileList" :key="file.full_path" class="file-item">
              <div class="file-info">
                <span class="file-name">{{ file.name }}</span>
                <span class="file-path">{{ file.full_path }}</span>
              </div>
              <button 
                class="download-file-button" 
                @click="downloadFile(file.full_path)"
                :disabled="downloadingFile === file.full_path"
              >
                <span v-if="downloadingFile === file.full_path">⏳</span>
                <span v-else>⬇️ Download</span>
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRoute } from 'vue-router'
import { useTokenNavigation } from '@/composables/useTokenNavigation'
import { API_CONFIG } from '@/config/api'

interface TorrentInfo {
  name: string
  progress: number
  size: number
  dlspeed: number
  upspeed: number
  eta: number
  content_path: string
  hash: string
  status: number
}

interface TorrentListResponse {
  torrents: TorrentInfo[]
}

interface FileInfo {
  name: string
  full_path: string
}

interface FileListResponse {
  files: FileInfo[]
}

const route = useRoute()
const { getRouteWithToken } = useTokenNavigation()

const torrents = ref<TorrentInfo[]>([])
const loading = ref(false)
const error = ref('')
const deleting = ref<string | null>(null)
const deleteMessage = ref('')
const showFileModal = ref(false)
const selectedTorrent = ref<TorrentInfo | null>(null)
const fileList = ref<FileInfo[]>([])
const loadingFiles = ref<string | null>(null)
const loadingFileList = ref(false)
const fileListError = ref('')
const downloadingFile = ref<string | null>(null)

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

const downloadingCount = computed(() => 
  torrents.value.filter(t => t.status === 1).length
)

const completedCount = computed(() => 
  torrents.value.filter(t => t.status === 0).length
)

const errorCount = computed(() => 
  torrents.value.filter(t => t.status === 2).length
)

const fetchTorrentStatus = async () => {
  loading.value = true
  error.value = ''
  
  try {
    const apiUrl = buildApiUrl('/torrent-list')
    const response = await fetch(apiUrl)
    
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`)
    }
    
    const data: TorrentListResponse = await response.json()
    torrents.value = data.torrents || []
    
  } catch (err) {
    error.value = 'Failed to fetch torrent status. Make sure the backend is running on localhost:3000.'
    console.error('Torrent status error:', err)
  } finally {
    loading.value = false
  }
}

const refreshStatus = () => {
  fetchTorrentStatus()
}

const getStatusText = (status: number): string => {
  switch (status) {
    case 0: return 'Completed'
    case 1: return 'Downloading'
    case 2: return 'Error'
    default: return 'Unknown'
  }
}

const getStatusClass = (status: number): string => {
  switch (status) {
    case 0: return 'status-completed'
    case 1: return 'status-downloading'
    case 2: return 'status-error'
    default: return 'status-unknown'
  }
}

const getProgressClass = (status: number): string => {
  switch (status) {
    case 0: return 'progress-completed'
    case 1: return 'progress-downloading'
    case 2: return 'progress-error'
    default: return 'progress-unknown'
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

const formatSpeed = (bytesPerSecond: number): string => {
  if (bytesPerSecond === 0) return '0 B/s'
  
  const units = ['B/s', 'KB/s', 'MB/s', 'GB/s']
  let speed = bytesPerSecond
  let unitIndex = 0
  
  while (speed >= 1024 && unitIndex < units.length - 1) {
    speed /= 1024
    unitIndex++
  }
  
  return `${speed.toFixed(1)} ${units[unitIndex]}`
}

const formatETA = (seconds: number, status: number): string => {
  if (status === 0) return 'Completed'
  if (status === 2) return 'Error'
  if (seconds === 0 || seconds === Infinity) return 'Unknown'
  
  const hours = Math.floor(seconds / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)
  const secs = seconds % 60
  
  if (hours > 0) {
    return `${hours}h ${minutes}m`
  } else if (minutes > 0) {
    return `${minutes}m ${secs}s`
  } else {
    return `${secs}s`
  }
}

const deleteTorrent = async (torrent: TorrentInfo) => {
  if (!confirm(`Are you sure you want to stop and remove "${torrent.name}"?`)) {
    return
  }
  
  deleting.value = torrent.hash
  deleteMessage.value = ''
  
  try {
    const apiUrl = buildApiUrl('/stop', { hash: torrent.hash })
    const response = await fetch(apiUrl)
    
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`)
    }
    
    const data = await response.json()
    
    if (data.status === 'success') {
      deleteMessage.value = data.message || 'Torrent stopped successfully!'
      // Remove the torrent from the list immediately
      torrents.value = torrents.value.filter(t => t.hash !== torrent.hash)
      // Clear the success message after 3 seconds
      setTimeout(() => {
        deleteMessage.value = ''
      }, 3000)
    } else {
      deleteMessage.value = 'Failed to stop torrent. Please try again.'
    }
  } catch (err) {
    deleteMessage.value = 'Failed to stop torrent. Make sure the backend is running.'
    console.error('Delete torrent error:', err)
  } finally {
    deleting.value = null
  }
}

const viewFiles = async (torrent: TorrentInfo) => {
  selectedTorrent.value = torrent
  showFileModal.value = true
  loadingFiles.value = torrent.hash
  loadingFileList.value = true
  fileListError.value = ''
  fileList.value = []
  
  try {
    const apiUrl = buildApiUrl('/list', { path: torrent.content_path })
    const response = await fetch(apiUrl)
    
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`)
    }
    
    const data: FileListResponse = await response.json()
    fileList.value = data.files || []
    
  } catch (err) {
    fileListError.value = 'Failed to load file list. Make sure the backend is running.'
    console.error('File list error:', err)
  } finally {
    loadingFiles.value = null
    loadingFileList.value = false
  }
}

const closeFileModal = () => {
  showFileModal.value = false
  selectedTorrent.value = null
  fileList.value = []
  fileListError.value = ''
}

const downloadFile = async (fullPath: string) => {
  downloadingFile.value = fullPath
  
  try {
    const token = route.query.token as string
    const baseUrl = `${API_CONFIG.BASE_URL}/sync/${fullPath}`
    const downloadUrl = token ? `${baseUrl}?token=${encodeURIComponent(token)}` : baseUrl
    
    // Create a temporary link element to trigger download
    const link = document.createElement('a')
    link.href = downloadUrl
    link.download = fullPath.split('/').pop() || 'download'
    link.target = '_blank'
    
    // Append to body, click, and remove
    document.body.appendChild(link)
    link.click()
    document.body.removeChild(link)
    
  } catch (err) {
    console.error('Download error:', err)
  } finally {
    downloadingFile.value = null
  }
}

const copyToClipboard = async (text: string) => {
  try {
    await navigator.clipboard.writeText(text)
    console.log('Hash copied to clipboard:', text)
  } catch (err) {
    console.error('Failed to copy hash to clipboard:', err)
  }
}

onMounted(() => {
  fetchTorrentStatus()
  
  // Auto-refresh every 5 seconds
  const interval = setInterval(fetchTorrentStatus, 5000)
  
  // Cleanup interval on component unmount
  return () => clearInterval(interval)
})
</script>

<style scoped>
.torrent-status {
  max-width: 1200px;
  margin: 0 auto;
  padding: 2rem;
}

.status-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 2rem;
}

.status-header h1 {
  color: var(--color-heading);
  font-size: 2.5rem;
  margin: 0;
}

.refresh-button {
  padding: 0.75rem 1.5rem;
  background-color: var(--color-background-mute);
  color: var(--color-text);
  border: 2px solid var(--color-border);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.3s;
  font-size: 1rem;
}

.refresh-button:hover:not(:disabled) {
  background-color: var(--color-background-soft);
  border-color: var(--color-border-hover);
}

.refresh-button:disabled {
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

.no-torrents {
  text-align: center;
  padding: 3rem;
  color: var(--color-text-light);
}

.no-torrents h2 {
  color: var(--color-heading);
  margin-bottom: 1rem;
}

.search-link {
  display: inline-block;
  margin-top: 1rem;
  padding: 0.75rem 1.5rem;
  background: var(--color-background-mute);
  color: var(--color-text);
  text-decoration: none;
  border-radius: 8px;
  border: 2px solid var(--color-border);
  transition: all 0.3s;
}

.search-link:hover {
  background: var(--color-background-soft);
  border-color: var(--color-border-hover);
}

.status-summary {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 1rem;
  margin-bottom: 2rem;
}

.summary-card {
  background: var(--color-background-mute);
  padding: 1.5rem;
  border-radius: 12px;
  border: 1px solid var(--color-border);
  text-align: center;
}

.summary-card h3 {
  color: var(--color-heading);
  margin: 0 0 0.5rem 0;
  font-size: 1rem;
}

.summary-card .count {
  font-size: 2rem;
  font-weight: 700;
  color: var(--color-heading);
}

.torrent-list {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.torrent-item {
  background: var(--color-background-mute);
  border: 1px solid var(--color-border);
  border-radius: 12px;
  padding: 1.5rem;
  transition: all 0.3s;
}

.torrent-item:hover {
  border-color: var(--color-border-hover);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.torrent-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 1rem;
}

.torrent-info {
  flex: 1;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.torrent-name {
  color: var(--color-heading);
  font-size: 1.2rem;
  margin: 0 0 0.5rem 0;
  line-height: 1.4;
}

.torrent-meta {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  font-size: 0.9rem;
  color: var(--color-text-light);
}

.torrent-path {
  font-family: monospace;
  font-size: 0.8rem;
}

.status-badge {
  padding: 0.5rem 1rem;
  border-radius: 20px;
  font-size: 0.8rem;
  font-weight: 600;
  white-space: nowrap;
}

.status-completed {
  background: #dcfce7;
  color: #166534;
}

.status-downloading {
  background: #dbeafe;
  color: #1e40af;
}

.status-error {
  background: #fee2e2;
  color: #dc2626;
}

.progress-section {
  display: flex;
  align-items: center;
  gap: 1rem;
  margin-bottom: 1rem;
}

.progress-bar {
  flex: 1;
  height: 8px;
  background: var(--color-border);
  border-radius: 4px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  transition: width 0.3s ease;
}

.progress-completed {
  background: #10b981;
}

.progress-downloading {
  background: #3b82f6;
}

.progress-error {
  background: #ef4444;
}

.progress-text {
  min-width: 50px;
  text-align: right;
  font-weight: 600;
  color: var(--color-heading);
}

.torrent-stats {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.stat-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 1rem;
}

.stat-item {
  display: flex;
  justify-content: space-between;
  font-size: 0.9rem;
}

.stat-label {
  color: var(--color-text-light);
}

.stat-value {
  font-weight: 600;
  color: var(--color-text);
}

.stat-value.download {
  color: #059669;
}

.stat-value.upload {
  color: #dc2626;
}

.stat-value.hash {
  cursor: pointer;
  font-family: monospace;
  font-size: 0.8rem;
}

.stat-value.hash:hover {
  text-decoration: underline;
}

.delete-button {
  padding: 0.5rem;
  background: #ef4444;
  color: white;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.3s;
  font-size: 1rem;
  min-width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.delete-button:hover:not(:disabled) {
  background: #dc2626;
  transform: scale(1.05);
}

.delete-button:disabled {
  opacity: 0.6;
  cursor: not-allowed;
  transform: none;
}

.view-files-button {
  padding: 0.5rem;
  background: #3b82f6;
  color: white;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.3s;
  font-size: 1rem;
  min-width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.view-files-button:hover:not(:disabled) {
  background: #2563eb;
  transform: scale(1.05);
}

.view-files-button:disabled {
  opacity: 0.6;
  cursor: not-allowed;
  transform: none;
}

.delete-message {
  background-color: #dcfce7;
  color: #166534;
  padding: 1rem;
  border-radius: 8px;
  margin-bottom: 2rem;
  text-align: center;
  border: 1px solid #bbf7d0;
}

/* Modal Styles */
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 1rem;
}

.modal-content {
  background: var(--color-background);
  border-radius: 12px;
  border: 1px solid var(--color-border);
  max-width: 800px;
  width: 100%;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1.5rem;
  border-bottom: 1px solid var(--color-border);
}

.modal-header h3 {
  color: var(--color-heading);
  margin: 0;
  font-size: 1.2rem;
}

.close-button {
  background: none;
  border: none;
  font-size: 1.5rem;
  cursor: pointer;
  color: var(--color-text-light);
  padding: 0.5rem;
  border-radius: 6px;
  transition: all 0.3s;
}

.close-button:hover {
  background: var(--color-background-soft);
  color: var(--color-text);
}

.modal-body {
  flex: 1;
  overflow-y: auto;
  padding: 1.5rem;
}

.no-files {
  text-align: center;
  color: var(--color-text-light);
  padding: 2rem;
}

.file-list {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.file-item {
  background: var(--color-background-mute);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  padding: 1rem;
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 1rem;
  transition: all 0.3s;
}

.file-item:hover {
  border-color: var(--color-border-hover);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.file-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  min-width: 0;
}

.file-name {
  font-weight: 600;
  color: var(--color-heading);
  word-break: break-word;
}

.file-path {
  font-size: 0.8rem;
  color: var(--color-text-light);
  font-family: monospace;
  word-break: break-all;
}

.download-file-button {
  padding: 0.5rem 1rem;
  background: #10b981;
  color: white;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.3s;
  font-size: 0.9rem;
  white-space: nowrap;
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.download-file-button:hover:not(:disabled) {
  background: #059669;
  transform: translateY(-1px);
}

.download-file-button:disabled {
  opacity: 0.6;
  cursor: not-allowed;
  transform: none;
}

@media (max-width: 768px) {
  .torrent-status {
    padding: 1rem;
  }
  
  .status-header {
    flex-direction: column;
    gap: 1rem;
    text-align: center;
  }
  
  .status-header h1 {
    font-size: 2rem;
  }
  
  .status-summary {
    grid-template-columns: repeat(2, 1fr);
  }
  
  .torrent-header {
    flex-direction: column;
    gap: 1rem;
  }
  
  .header-actions {
    flex-direction: row;
    align-self: flex-end;
  }
  
  .stat-row {
    grid-template-columns: 1fr;
  }
  
  .torrent-meta {
    gap: 0.5rem;
  }
  
  .modal-content {
    max-height: 90vh;
    margin: 0.5rem;
  }
  
  .file-item {
    flex-direction: column;
    align-items: stretch;
    gap: 0.75rem;
  }
  
  .download-file-button {
    align-self: flex-end;
  }
}
</style> 