<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const emit = defineEmits(['complete', 'next', 'back'])

interface EnvStatus {
  node: { installed: boolean; version: string }
  npm: { installed: boolean; version: string }
  openclaw: { installed: boolean; version: string }
  gateway: { running: boolean; port: number } | null
}

const envStatus = ref<EnvStatus | null>(null)
const loading = ref(true)
const error = ref('')

async function checkEnvironment() {
  loading.value = true
  error.value = ''
  
  try {
    envStatus.value = await invoke('get_env_status')
  } catch (e) {
    error.value = `检测失败: ${e}`
  } finally {
    loading.value = false
  }
}

function proceed() {
  if (envStatus.value) {
    emit('complete', {
      nodeInstalled: envStatus.value.node.installed,
      openclawInstalled: envStatus.value.openclaw.installed,
    })
    emit('next')
  }
}

onMounted(checkEnvironment)
</script>

<template>
  <div class="env-detector">
    <h2>🔍 环境检测</h2>
    <p class="subtitle">检测您的系统环境</p>

    <div v-if="loading" class="loading">
      <div class="spinner"></div>
      <p>正在检测环境...</p>
    </div>

    <div v-else-if="error" class="error">
      <p>{{ error }}</p>
      <button class="btn" @click="checkEnvironment">重试</button>
    </div>

    <div v-else class="status-grid">
      <!-- Node.js -->
      <div class="status-card" :class="{ success: envStatus?.node.installed, error: !envStatus?.node.installed }">
        <div class="status-icon">{{ envStatus?.node.installed ? '✅' : '❌' }}</div>
        <div class="status-info">
          <h3>Node.js</h3>
          <p v-if="envStatus?.node.installed">v{{ envStatus.node.version }}</p>
          <p v-else>未安装</p>
        </div>
      </div>

      <!-- npm -->
      <div class="status-card" :class="{ success: envStatus?.npm.installed, error: !envStatus?.npm.installed }">
        <div class="status-icon">{{ envStatus?.npm.installed ? '✅' : '❌' }}</div>
        <div class="status-info">
          <h3>npm</h3>
          <p v-if="envStatus?.npm.installed">v{{ envStatus.npm.version }}</p>
          <p v-else>未安装</p>
        </div>
      </div>

      <!-- OpenClaw -->
      <div class="status-card" :class="{ success: envStatus?.openclaw.installed, error: !envStatus?.openclaw.installed }">
        <div class="status-icon">{{ envStatus?.openclaw.installed ? '✅' : '❌' }}</div>
        <div class="status-info">
          <h3>OpenClaw</h3>
          <p v-if="envStatus?.openclaw.installed">v{{ envStatus.openclaw.version }}</p>
          <p v-else>未安装</p>
        </div>
      </div>

      <!-- Gateway -->
      <div class="status-card" :class="{ success: envStatus?.gateway?.running, error: !envStatus?.gateway?.running }">
        <div class="status-icon">{{ envStatus?.gateway?.running ? '✅' : '⏳' }}</div>
        <div class="status-info">
          <h3>Gateway</h3>
          <p v-if="envStatus?.gateway?.running">运行中 (端口 {{ envStatus.gateway.port }})</p>
          <p v-else>未运行</p>
        </div>
      </div>
    </div>

    <div class="actions">
      <button class="btn btn-secondary" @click="checkEnvironment">
        🔄 重新检测
      </button>
      <button class="btn btn-primary" @click="proceed">
        继续 →
      </button>
    </div>
  </div>
</template>

<style scoped>
.env-detector {
  max-width: 600px;
  margin: 0 auto;
}

h2 {
  font-size: 24px;
  margin-bottom: 8px;
}

.subtitle {
  color: var(--text-secondary);
  margin-bottom: 32px;
}

.loading {
  text-align: center;
  padding: 48px;
}

.spinner {
  width: 40px;
  height: 40px;
  border: 3px solid var(--bg-card);
  border-top-color: var(--cyan);
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin: 0 auto 16px;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.status-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 16px;
  margin-bottom: 32px;
}

.status-card {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 20px;
  background: var(--bg-card);
  border-radius: 12px;
  border: 2px solid transparent;
  transition: all 0.2s;
}

.status-card.success {
  border-color: var(--green);
}

.status-card.error {
  border-color: var(--red);
  opacity: 0.7;
}

.status-icon {
  font-size: 32px;
}

.status-info h3 {
  font-size: 16px;
  margin-bottom: 4px;
}

.status-info p {
  font-size: 14px;
  color: var(--text-secondary);
}

.actions {
  display: flex;
  justify-content: space-between;
}

.btn {
  padding: 12px 24px;
  border-radius: 8px;
  font-weight: 600;
  cursor: pointer;
  border: none;
  transition: all 0.2s;
}

.btn-primary {
  background: var(--cyan);
  color: var(--bg-deep);
}

.btn-secondary {
  background: var(--bg-card);
  color: var(--text-primary);
}
</style>
