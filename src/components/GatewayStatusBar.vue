<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const props = defineProps<{
  compact?: boolean
}>()

interface GatewayHealth {
  running: boolean
  port: number
  version: string
  uptime_sec: number
}

const health = ref<GatewayHealth | null>(null)
const loading = ref(true)
const error = ref('')
const showLogs = ref(false)
const logs = ref<string[]>([])

let pollInterval: number | null = null

async function checkHealth() {
  loading.value = true
  try {
    health.value = await invoke('gateway_health', { port: 18789 })
    error.value = ''
  } catch (e) {
    health.value = { running: false, port: 18789, version: '', uptime_sec: 0 }
    error.value = ''
  } finally {
    loading.value = false
  }
}

async function startGateway() {
  try {
    await invoke('gateway_start', { port: 18789 })
    await checkHealth()
  } catch (e) {
    error.value = `启动失败: ${e}`
  }
}

async function stopGateway() {
  try {
    await invoke('gateway_stop')
    await checkHealth()
  } catch (e) {
    error.value = `停止失败: ${e}`
  }
}

async function restartGateway() {
  try {
    await invoke('gateway_restart', { port: 18789 })
    await checkHealth()
  } catch (e) {
    error.value = `重启失败: ${e}`
  }
}

async function viewLogs() {
  try {
    logs.value = await invoke('gateway_logs', { tail: 50 })
    showLogs.value = true
  } catch (e) {
    logs.value = [`获取日志失败: ${e}`]
  }
}

function formatUptime(sec: number): string {
  if (sec < 60) return `${sec}秒`
  if (sec < 3600) return `${Math.floor(sec / 60)}分`
  if (sec < 86400) return `${Math.floor(sec / 3600)}小时`
  return `${Math.floor(sec / 86400)}天`
}

onMounted(() => {
  checkHealth()
  // Poll every 10 seconds
  pollInterval = window.setInterval(checkHealth, 10000)
})

onUnmounted(() => {
  if (pollInterval) clearInterval(pollInterval)
})
</script>

<template>
  <div class="gateway-status-bar" :class="{ compact: props.compact }">
    <!-- Status Indicator -->
    <div class="status-left">
      <div class="logo">🦞</div>
      <div class="status-info">
        <span class="version">OpenClaw {{ health?.version || '...' }}</span>
        <span v-if="!props.compact" class="divider">|</span>
        <span v-if="!props.compact" class="port">端口: {{ health?.port || 18789 }}</span>
      </div>
    </div>

    <!-- Status & Actions -->
    <div class="status-center">
      <div class="status-badge" :class="{ running: health?.running }">
        <span class="dot"></span>
        {{ health?.running ? '运行中' : '已停止' }}
      </div>
      
      <span v-if="health?.running && !props.compact" class="uptime">
        ↑ {{ formatUptime(health.uptime_sec) }}
      </span>
    </div>

    <!-- Action Buttons -->
    <div class="status-actions">
      <button 
        v-if="!health?.running"
        class="btn btn-success btn-sm"
        @click="startGateway"
      >
        启动
      </button>
      
      <template v-else>
        <button 
          v-if="!props.compact"
          class="btn btn-secondary btn-sm"
          @click="restartGateway"
        >
          重启
        </button>
        <button 
          class="btn btn-danger btn-sm"
          @click="stopGateway"
        >
          停止
        </button>
      </template>
      
      <button 
        v-if="!props.compact && health?.running"
        class="btn btn-ghost btn-sm"
        @click="viewLogs"
      >
        日志
      </button>
    </div>

    <!-- Logs Modal -->
    <div v-if="showLogs" class="logs-modal" @click.self="showLogs = false">
      <div class="logs-content">
        <div class="logs-header">
          <h3>Gateway 日志</h3>
          <button class="close-btn" @click="showLogs = false">×</button>
        </div>
        <div class="logs-body">
          <div v-for="(log, i) in logs" :key="i" class="log-line">{{ log }}</div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.gateway-status-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 20px;
  background: var(--bg-card);
  border-radius: 8px;
  border: 1px solid var(--border-color);
}

.gateway-status-bar.compact {
  padding: 8px 12px;
}

.status-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.logo {
  font-size: 24px;
}

.status-info {
  display: flex;
  align-items: center;
  gap: 8px;
}

.version {
  font-weight: 600;
  font-size: 14px;
}

.divider {
  color: var(--text-secondary);
}

.port {
  color: var(--text-secondary);
  font-size: 13px;
}

.status-center {
  display: flex;
  align-items: center;
  gap: 12px;
}

.status-badge {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 12px;
  background: rgba(239, 68, 68, 0.1);
  border-radius: 12px;
  font-size: 13px;
  color: var(--red);
}

.status-badge.running {
  background: rgba(34, 197, 94, 0.1);
  color: var(--green);
}

.dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: currentColor;
}

.uptime {
  font-size: 12px;
  color: var(--text-secondary);
}

.status-actions {
  display: flex;
  gap: 8px;
}

.btn {
  padding: 6px 12px;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  border: none;
  transition: all 0.2s;
}

.btn-sm {
  padding: 4px 10px;
  font-size: 12px;
}

.btn-success {
  background: var(--green);
  color: white;
}

.btn-danger {
  background: var(--red);
  color: white;
}

.btn-secondary {
  background: var(--bg-base);
  color: var(--text-primary);
}

.btn-ghost {
  background: transparent;
  color: var(--text-secondary);
}

.btn:hover {
  filter: brightness(1.1);
}

/* Logs Modal */
.logs-modal {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.logs-content {
  background: var(--bg-base);
  border-radius: 12px;
  width: 80%;
  max-width: 800px;
  max-height: 70vh;
  overflow: hidden;
}

.logs-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border-color);
}

.logs-header h3 {
  margin: 0;
}

.close-btn {
  background: none;
  border: none;
  font-size: 24px;
  cursor: pointer;
  color: var(--text-secondary);
}

.logs-body {
  padding: 16px;
  max-height: 500px;
  overflow-y: auto;
  font-family: monospace;
  font-size: 12px;
  background: var(--bg-deep);
}

.log-line {
  padding: 2px 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
