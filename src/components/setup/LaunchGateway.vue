<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useRouter } from 'vue-router'

const emit = defineEmits(['complete', 'back'])
const router = useRouter()

// 双状态：Proxy 网关 + OpenClaw 引擎
type ServiceStatus = 'waiting' | 'starting' | 'running' | 'error'

const proxyStatus = ref<ServiceStatus>('waiting')
const openclawStatus = ref<ServiceStatus>('waiting')
const proxyMessage = ref('🛡️ 视控舱安全网关 (Port: 18788)')
const openclawMessage = ref('🦞 OpenClaw 核心引擎 (Port: 18789)')
const errorMessage = ref('')

async function checkStatus() {
  // 检查 Proxy 状态
  try {
    const status: any = await invoke('get_proxy_status')
    proxyStatus.value = status.running ? 'running' : 'waiting'
    if (status.running) {
      proxyMessage.value = `🛡️ 网关运行中 ($${status.total_cost?.toFixed(4) ?? '0.00'})`
    }
  } catch {
    proxyStatus.value = 'waiting'
    proxyMessage.value = '🛡️ 视控舱安全网关 (Port: 18788)'
  }

  // 检查 Gateway 状态
  try {
    const health: any = await invoke('gateway_health', { port: 18789 })
    openclawStatus.value = health.running ? 'running' : 'waiting'
    if (health.running) {
      openclawMessage.value = '🦞 OpenClaw 运行中'
    }
  } catch {
    openclawStatus.value = 'waiting'
    openclawMessage.value = '🦞 OpenClaw 核心引擎 (Port: 18789)'
  }
}

async function startBoth() {
  errorMessage.value = ''
  
  // ========== 阶段1: 配置并启动 Proxy ==========
  proxyStatus.value = 'starting'
  proxyMessage.value = '⚙️ 正在劫持配置并启动安全网关...'
  
  try {
    // 1. 强制劫持 OpenClaw 的 LLM Base URL 指向本地 Proxy
    await invoke('configure_openclaw_proxy', { proxyPort: 18788 })
    
    // 2. 启动本地 API 代理服务器
    await invoke('start_proxy', { port: 18788 })
    
    proxyStatus.value = 'running'
    proxyMessage.value = '✅ 安全网关已接管 (Port: 18788)'
  } catch (e) {
    proxyStatus.value = 'error'
    proxyMessage.value = `❌ 网关启动失败`
    errorMessage.value = `Proxy 启动失败: ${e}`
    return
  }

  // ========== 阶段2: 启动 OpenClaw ==========
  openclawStatus.value = 'starting'
  openclawMessage.value = '⚙️ 正在启动 OpenClaw 核心...'
  
  try {
    await invoke('start_gateway', { port: 18789 })
    openclawStatus.value = 'running'
    openclawMessage.value = '✅ OpenClaw 已连接'
  } catch (e) {
    openclawStatus.value = 'error'
    openclawMessage.value = `❌ 引擎启动失败`
    errorMessage.value = `Gateway 启动失败: ${e}`
    return
  }

  // ========== 阶段3: 跳转到 Dashboard ==========
  setTimeout(() => {
    router.push('/dashboard')
  }, 1500)
}

function finish() {
  router.push('/dashboard')
}

function getStatusIcon(status: ServiceStatus): string {
  switch (status) {
    case 'waiting': return '⏳'
    case 'starting': return '⚙️'
    case 'running': return '✅'
    case 'error': return '❌'
  }
}

function getStatusClass(status: ServiceStatus): string {
  switch (status) {
    case 'waiting': return 'status-waiting'
    case 'starting': return 'status-starting'
    case 'running': return 'status-running'
    case 'error': return 'status-error'
  }
}

onMounted(checkStatus)
</script>

<template>
  <div class="launch-gateway">
    <h2>🚀 启动引擎</h2>
    <p class="subtitle">双重启动：安全网关 → 核心引擎</p>

    <!-- 双重状态灯 -->
    <div class="dual-status">
      <!-- Proxy 状态 -->
      <div class="service-card" :class="getStatusClass(proxyStatus)">
        <div class="service-header">
          <span class="service-icon">{{ getStatusIcon(proxyStatus) }}</span>
          <span class="service-label">{{ proxyMessage }}</span>
        </div>
        <div class="status-badge" :class="getStatusClass(proxyStatus)">
          {{ proxyStatus === 'waiting' ? '等待启动' :
             proxyStatus === 'starting' ? '启动中...' :
             proxyStatus === 'running' ? '运行中' : '启动失败' }}
        </div>
      </div>

      <!-- OpenClaw 状态 -->
      <div class="service-card" :class="getStatusClass(openclawStatus)">
        <div class="service-header">
          <span class="service-icon">{{ getStatusIcon(openclawStatus) }}</span>
          <span class="service-label">{{ openclawMessage }}</span>
        </div>
        <div class="status-badge" :class="getStatusClass(openclawStatus)">
          {{ openclawStatus === 'waiting' ? '等待启动' :
             openclawStatus === 'starting' ? '启动中...' :
             openclawStatus === 'running' ? '运行中' : '启动失败' }}
        </div>
      </div>
    </div>

    <!-- 错误提示 -->
    <div v-if="errorMessage" class="error-panel">
      <span class="error-icon">⚠️</span>
      <span class="error-text">{{ errorMessage }}</span>
    </div>

    <!-- 操作区 -->
    <div class="actions">
      <!-- 启动按钮 -->
      <button
        v-if="proxyStatus !== 'running' || openclawStatus !== 'running'"
        class="btn btn-primary btn-large"
        :disabled="proxyStatus === 'starting' || openclawStatus === 'starting'"
        @click="startBoth"
      >
        <span v-if="proxyStatus === 'starting' || openclawStatus === 'starting'">
          ⚙️ 启动中...
        </span>
        <span v-else>
          🛡️ 启动安全网关 & 引擎
        </span>
      </button>

      <!-- 已运行状态 -->
      <div v-else class="success-panel">
        <div class="success-icon">🎉</div>
        <p class="success-text">双重启动成功！准备进入控制台...</p>
        <div class="success-actions">
          <button class="btn btn-secondary" @click="checkStatus">
            🔄 刷新状态
          </button>
          <button class="btn btn-success btn-large" @click="finish">
            🚀 进入控制台
          </button>
        </div>
      </div>
    </div>

    <!-- 安全说明 -->
    <div class="security-note">
      <p>🔒 安全机制：所有 LLM 请求都会经过本地网关监控，拦截高危操作并追踪费用</p>
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
  max-width: 560px;
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

/* 双重状态卡片 */
.dual-status {
  display: flex;
  flex-direction: column;
  gap: 16px;
  margin-bottom: 24px;
}

.service-card {
  padding: 20px;
  background: var(--bg-card);
  border-radius: 12px;
  border: 2px solid var(--border);
  transition: all 0.3s;
}

.service-card.status-running {
  border-color: var(--green);
  background: linear-gradient(135deg, rgba(34, 197, 94, 0.1) 0%, var(--bg-card) 100%);
}

.service-card.status-starting {
  border-color: var(--yellow);
  background: linear-gradient(135deg, rgba(234, 179, 8, 0.1) 0%, var(--bg-card) 100%);
}

.service-card.status-error {
  border-color: var(--red);
  background: linear-gradient(135deg, rgba(239, 68, 68, 0.1) 0%, var(--bg-card) 100%);
}

.service-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 8px;
}

.service-icon {
  font-size: 28px;
}

.service-label {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
}

.status-badge {
  display: inline-block;
  padding: 4px 12px;
  border-radius: 20px;
  font-size: 12px;
  font-weight: 500;
}

.status-badge.status-waiting {
  background: rgba(255, 255, 255, 0.1);
  color: var(--text-secondary);
}

.status-badge.status-starting {
  background: rgba(234, 179, 8, 0.2);
  color: var(--yellow);
}

.status-badge.status-running {
  background: rgba(34, 197, 94, 0.2);
  color: var(--green);
}

.status-badge.status-error {
  background: rgba(239, 68, 68, 0.2);
  color: var(--red);
}

/* 错误提示 */
.error-panel {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid var(--red);
  border-radius: 8px;
  margin-bottom: 24px;
  text-align: left;
}

.error-icon {
  font-size: 20px;
}

.error-text {
  color: var(--red);
  font-size: 14px;
}

/* 操作按钮 */
.actions {
  margin-bottom: 24px;
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
  opacity: 0.6;
  cursor: not-allowed;
}

.btn-primary {
  background: linear-gradient(135deg, var(--cyan) 0%, #0891b2 100%);
  color: var(--bg-deep);
}

.btn-primary:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(34, 211, 238, 0.4);
}

.btn-success {
  background: var(--green);
  color: white;
}

.btn-secondary {
  background: var(--bg-card);
  color: var(--text-primary);
  border: 1px solid var(--border);
}

/* 成功状态 */
.success-panel {
  padding: 24px;
  background: linear-gradient(135deg, rgba(34, 197, 94, 0.1) 0%, var(--bg-card) 100%);
  border-radius: 12px;
  border: 1px solid var(--green);
}

.success-icon {
  font-size: 48px;
  margin-bottom: 12px;
}

.success-text {
  font-size: 16px;
  color: var(--green);
  margin-bottom: 20px;
}

.success-actions {
  display: flex;
  gap: 12px;
  justify-content: center;
}

/* 安全说明 */
.security-note {
  padding: 12px 16px;
  background: rgba(34, 211, 238, 0.1);
  border-radius: 8px;
  margin-bottom: 24px;
}

.security-note p {
  margin: 0;
  font-size: 13px;
  color: var(--cyan);
}

.footer-actions {
  display: flex;
  justify-content: flex-start;
}
</style>
