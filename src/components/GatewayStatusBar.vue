<template>
  <div class="gateway-status-bar" :class="statusClass">
    <div class="status-left">
      <div class="status-icon">
        <span v-if="loading" class="spinner">⏳</span>
        <span v-else-if="running">🟢</span>
        <span v-else>🔴</span>
      </div>
      <div class="status-info">
        <span class="status-label">OpenClaw {{ version || 'v?.?.?' }}</span>
        <span class="status-text">
          Gateway: {{ running ? `Running on port ${port}` : 'Stopped' }}
        </span>
      </div>
    </div>
    
    <div class="status-center">
      <span v-if="uptime" class="uptime">↑ {{ formatUptime(uptime) }}</span>
    </div>
    
    <div class="status-right">
      <button v-if="!running" @click="start" class="btn-action start" :disabled="loading">
        {{ loading ? 'Starting...' : 'Start' }}
      </button>
      <button v-else @click="restart" class="btn-action restart" :disabled="loading">
        Restart
      </button>
      <button v-if="running" @click="stop" class="btn-action stop" :disabled="loading">
        Stop
      </button>
      <button @click="showLogs" class="btn-action logs">
        Logs
      </button>
    </div>
    
    <!-- Logs Modal -->
    <div v-if="logsVisible" class="logs-modal">
      <div class="logs-content">
        <div class="logs-header">
          <h3>Gateway Logs</h3>
          <button @click="logsVisible = false" class="btn-close">×</button>
        </div>
        <div class="logs-body">
          <div v-for="(log, i) in logs" :key="i" class="log-line">{{ log }}</div>
          <div v-if="logs.length === 0" class="log-empty">No logs available</div>
        </div>
        <div class="logs-footer">
          <button @click="refreshLogs" class="btn-refresh">Refresh</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const running = ref(false)
const loading = ref(false)
const port = ref(18789)
const uptime = ref<number | null>(null)
const version = ref('')
const logs = ref<string[]>([])
const logsVisible = ref(false)

let pollInterval: number | null = null

const statusClass = computed(() => ({
  'status-running': running.value,
  'status-stopped': !running.value,
  'status-loading': loading.value,
}))

onMounted(async () => {
  await fetchStatus()
  // Poll every 5 seconds
  pollInterval = window.setInterval(fetchStatus, 5000)
})

onUnmounted(() => {
  if (pollInterval) {
    clearInterval(pollInterval)
  }
})

async function fetchStatus() {
  try {
    const status = await invoke<any>('gateway_status')
    running.value = status.running
    port.value = status.port
    uptime.value = status.uptime_secs
    version.value = status.version || ''
  } catch (error) {
    console.error('Failed to fetch gateway status:', error)
    running.value = false
  }
}

async function start() {
  loading.value = true
  try {
    await invoke('start_gateway', { port: port.value })
    await fetchStatus()
  } catch (error) {
    console.error('Failed to start gateway:', error)
  } finally {
    loading.value = false
  }
}

async function stop() {
  loading.value = true
  try {
    await invoke('stop_gateway')
    await fetchStatus()
  } catch (error) {
    console.error('Failed to stop gateway:', error)
  } finally {
    loading.value = false
  }
}

async function restart() {
  loading.value = true
  try {
    await invoke('restart_gateway')
    await fetchStatus()
  } catch (error) {
    console.error('Failed to restart gateway:', error)
  } finally {
    loading.value = false
  }
}

async function showLogs() {
  logsVisible.value = true
  await refreshLogs()
}

async function refreshLogs() {
  try {
    logs.value = await invoke('gateway_logs', { tail: 100 })
  } catch (error) {
    console.error('Failed to fetch logs:', error)
    logs.value = ['Failed to load logs']
  }
}

function formatUptime(secs: number): string {
  if (secs < 60) return `${secs}s`
  if (secs < 3600) return `${Math.floor(secs / 60)}m ${secs % 60}s`
  const hours = Math.floor(secs / 3600)
  const mins = Math.floor((secs % 3600) / 60)
  return `${hours}h ${mins}m`
}
</script>

<style scoped>
.gateway-status-bar {
  display: flex;
  align-items: center;
  gap: 1.5rem;
  padding: 0.75rem 1.25rem;
  background: rgba(255, 255, 255, 0.05);
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  font-size: 0.875rem;
}

.status-running {
  border-left: 3px solid #22c55e;
}

.status-stopped {
  border-left: 3px solid #ef4444;
}

.status-left {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.status-icon {
  font-size: 1rem;
}

.spinner {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.status-info {
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
}

.status-label {
  font-weight: 500;
  color: #e0e0e0;
}

.status-text {
  font-size: 0.75rem;
  color: #888;
}

.status-center {
  flex: 1;
  display: flex;
  justify-content: center;
}

.uptime {
  padding: 0.25rem 0.75rem;
  background: rgba(34, 197, 94, 0.1);
  border-radius: 4px;
  color: #22c55e;
  font-size: 0.75rem;
}

.status-right {
  display: flex;
  gap: 0.5rem;
}

.btn-action {
  padding: 0.5rem 1rem;
  border: none;
  border-radius: 6px;
  font-size: 0.75rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-action.start {
  background: #22c55e;
  color: white;
}

.btn-action.start:hover:not(:disabled) {
  background: #16a34a;
}

.btn-action.restart {
  background: rgba(234, 179, 8, 0.2);
  color: #eab308;
  border: 1px solid rgba(234, 179, 8, 0.3);
}

.btn-action.stop {
  background: rgba(239, 68, 68, 0.2);
  color: #ef4444;
  border: 1px solid rgba(239, 68, 68, 0.3);
}

.btn-action.logs {
  background: rgba(59, 130, 246, 0.2);
  color: #3b82f6;
  border: 1px solid rgba(59, 130, 246, 0.3);
}

.btn-action:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.logs-modal {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.8);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.logs-content {
  background: #1a1a2e;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 12px;
  width: 80%;
  max-width: 800px;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
}

.logs-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1rem 1.5rem;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
}

.logs-header h3 {
  margin: 0;
  font-size: 1rem;
}

.btn-close {
  background: none;
  border: none;
  color: #888;
  font-size: 1.5rem;
  cursor: pointer;
}

.logs-body {
  flex: 1;
  overflow-y: auto;
  padding: 1rem;
  background: rgba(0, 0, 0, 0.3);
  font-family: monospace;
  font-size: 0.75rem;
}

.log-line {
  padding: 0.25rem 0;
  color: #888;
  white-space: pre-wrap;
  word-break: break-all;
}

.log-empty {
  color: #666;
  text-align: center;
  padding: 2rem;
}

.logs-footer {
  padding: 0.75rem 1.5rem;
  border-top: 1px solid rgba(255, 255, 255, 0.1);
}

.btn-refresh {
  padding: 0.5rem 1rem;
  background: rgba(59, 130, 246, 0.2);
  border: 1px solid rgba(59, 130, 246, 0.3);
  border-radius: 6px;
  color: #3b82f6;
  cursor: pointer;
}
</style>