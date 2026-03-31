<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const emit = defineEmits(['complete', 'back'])

const status = ref<'idle' | 'starting' | 'running' | 'error'>('idle')
const message = ref('')
const error = ref('')

async function checkStatus() {
  try {
    const health = await invoke('gateway_health', { port: 18789 })
    const running = (health as any).running
    status.value = running ? 'running' : 'idle'
    message.value = running ? 'Gateway 运行中' : 'Gateway 未启动'
  } catch (e) {
    status.value = 'idle'
    message.value = 'Gateway 未启动'
  }
}

async function startGateway() {
  status.value = 'starting'
  message.value = '正在启动 Gateway...'
  error.value = ''

  try {
    await invoke('gateway_start', { port: 18789 })
    status.value = 'running'
    message.value = '🎉 Gateway 启动成功!'
  } catch (e) {
    status.value = 'error'
    error.value = `启动失败: ${e}`
  }
}

function finish() {
  emit('complete', { gatewayStarted: true })
}

onMounted(checkStatus)
</script>

<template>
  <div class="launch-gateway">
    <h2>🚀 启动 Gateway</h2>
    <p class="subtitle">启动 OpenClaw Gateway 服务</p>

    <!-- Status Display -->
    <div class="status-panel" :class="status">
      <div class="status-icon">
        <span v-if="status === 'idle'">⏳</span>
        <span v-else-if="status === 'starting'">⚙️</span>
        <span v-else-if="status === 'running'">✅</span>
        <span v-else>❌</span>
      </div>
      
      <div class="status-message">
        {{ message }}
      </div>
      
      <div v-if="error" class="error-message">
        {{ error }}
      </div>
    </div>

    <!-- Instructions -->
    <div v-if="status === 'idle'" class="instructions">
      <p>点击下方按钮启动 Gateway 服务</p>
      <p class="hint">Gateway 是 OpenClaw 的核心服务，负责管理所有 Agent</p>
    </div>

    <!-- Running Info -->
    <div v-if="status === 'running'" class="running-info">
      <div class="info-item">
        <span class="label">端口:</span>
        <span class="value">18789</span>
      </div>
      <div class="info-item">
        <span class="label">状态:</span>
        <span class="value success">运行中</span>
      </div>
    </div>

    <!-- Actions -->
    <div class="actions">
      <button 
        v-if="status !== 'running'"
        class="btn btn-primary btn-large"
        :disabled="status === 'starting'"
        @click="startGateway"
      >
        {{ status === 'starting' ? '启动中...' : '启动 Gateway' }}
      </button>
      
      <div v-if="status === 'running'" class="success-actions">
        <button class="btn btn-secondary" @click="checkStatus">
          🔄 刷新状态
        </button>
        <button class="btn btn-success btn-large" @click="finish">
          🚀 进入控制台
        </button>
      </div>
    </div>

    <div class="footer-actions">
      <button class="btn btn-secondary" @click="$emit('back')">
        ← 上一步
      </button>
    </div>
  </div>
</template>

<style scoped>
.launch-gateway {
  max-width: 500px;
  margin: 0 auto;
  text-align: center;
}

h2 {
  font-size: 24px;
  margin-bottom: 8px;
}

.subtitle {
  color: var(--text-secondary);
  margin-bottom: 32px;
}

.status-panel {
  padding: 48px;
  background: var(--bg-card);
  border-radius: 16px;
  margin-bottom: 24px;
}

.status-panel.running {
  border: 2px solid var(--green);
}

.status-panel.error {
  border: 2px solid var(--red);
}

.status-icon {
  font-size: 64px;
  margin-bottom: 16px;
}

.status-message {
  font-size: 20px;
  font-weight: 600;
  margin-bottom: 8px;
}

.error-message {
  color: var(--red);
  margin-top: 8px;
}

.instructions {
  margin-bottom: 24px;
}

.instructions p {
  margin-bottom: 8px;
}

.hint {
  font-size: 13px;
  color: var(--text-secondary);
}

.running-info {
  display: flex;
  justify-content: center;
  gap: 32px;
  margin-bottom: 24px;
}

.info-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.info-item .label {
  font-size: 12px;
  color: var(--text-secondary);
}

.info-item .value {
  font-size: 16px;
  font-weight: 600;
}

.info-item .value.success {
  color: var(--green);
}

.actions {
  margin-bottom: 24px;
}

.success-actions {
  display: flex;
  gap: 12px;
  justify-content: center;
}

.btn {
  padding: 12px 24px;
  border-radius: 8px;
  font-weight: 600;
  cursor: pointer;
  border: none;
  transition: all 0.2s;
}

.btn-large {
  padding: 16px 48px;
  font-size: 16px;
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-primary {
  background: var(--cyan);
  color: var(--bg-deep);
}

.btn-success {
  background: var(--green);
  color: white;
}

.btn-secondary {
  background: var(--bg-card);
  color: var(--text-primary);
}

.footer-actions {
  display: flex;
  justify-content: flex-start;
}
</style>
