<template>
  <div class="setup-wizard">
    <!-- Progress indicator -->
    <div class="setup-progress">
      <div 
        v-for="(step, index) in steps" 
        :key="step.id"
        class="step-indicator"
        :class="{ active: currentStep === index, completed: index < currentStep }"
      >
        <div class="step-number">{{ index + 1 }}</div>
        <div class="step-label">{{ step.label }}</div>
      </div>
    </div>

    <!-- Step content -->
    <div class="setup-content">
      <!-- Step 0: Environment Detection -->
      <div v-if="currentStep === 0" class="step-panel">
        <h2>🔍 Environment Detection</h2>
        <p class="step-desc">Checking your system for required dependencies...</p>
        
        <div class="env-list">
          <div v-for="item in envItems" :key="item.name" class="env-item">
            <div class="env-icon" :class="item.status">
              <span v-if="item.status === 'checking'">⏳</span>
              <span v-else-if="item.status === 'ok'">✅</span>
              <span v-else-if="item.status === 'missing'">❌</span>
              <span v-else>⚠️</span>
            </div>
            <div class="env-info">
              <div class="env-name">{{ item.name }}</div>
              <div class="env-version">{{ item.version || 'Not installed' }}</div>
            </div>
          </div>
        </div>
        
        <div class="step-actions">
          <button @click="startSetup" class="btn-primary" :disabled="isChecking">
            {{ isChecking ? 'Checking...' : 'Continue' }}
          </button>
        </div>
      </div>

      <!-- Step 1: Install Node.js -->
      <div v-if="currentStep === 1" class="step-panel">
        <h2>📦 Install Node.js</h2>
        <p class="step-desc">Node.js is required to run OpenClaw.</p>
        
        <div v-if="installProgress.step === 'install_node'" class="install-progress">
          <div class="progress-bar">
            <div class="progress-fill" :style="{ width: `${installProgress.percent}%` }"></div>
          </div>
          <div class="progress-message">{{ installProgress.message }}</div>
          <div v-if="installProgress.log_line" class="log-output">
            <code>{{ installProgress.log_line }}</code>
          </div>
        </div>
        
        <div class="step-actions">
          <button @click="skipStep" class="btn-secondary">Skip (I have Node.js)</button>
          <button @click="installNode" class="btn-primary" :disabled="isInstalling">
            {{ isInstalling ? 'Installing...' : 'Install Node.js v22' }}
          </button>
        </div>
      </div>

      <!-- Step 2: Install OpenClaw -->
      <div v-if="currentStep === 2" class="step-panel">
        <h2>🦞 Install OpenClaw</h2>
        <p class="step-desc">Installing the OpenClaw CLI...</p>
        
        <div class="mirror-option">
          <label>
            <input type="checkbox" v-model="useMirror" />
            Use China mirror (faster download in mainland China)
          </label>
        </div>
        
        <div v-if="installProgress.step === 'install_openclaw'" class="install-progress">
          <div class="progress-bar">
            <div class="progress-fill" :style="{ width: `${installProgress.percent}%` }"></div>
          </div>
          <div class="progress-message">{{ installProgress.message }}</div>
          <div v-if="installProgress.log_line" class="log-output">
            <code>{{ installProgress.log_line }}</code>
          </div>
        </div>
        
        <div class="step-actions">
          <button @click="installOpenClaw" class="btn-primary" :disabled="isInstalling">
            {{ isInstalling ? 'Installing...' : 'Install OpenClaw' }}
          </button>
        </div>
      </div>

      <!-- Step 3: Configure API Key -->
      <div v-if="currentStep === 3" class="step-panel">
        <h2>🔑 Configure API Key</h2>
        <p class="step-desc">Enter your AI provider API key to get started.</p>
        
        <div class="api-config">
          <div class="provider-tabs">
            <button 
              v-for="p in providers" 
              :key="p.id"
              :class="['provider-tab', { active: selectedProvider === p.id }]"
              @click="selectedProvider = p.id"
            >
              {{ p.name }}
            </button>
          </div>
          
          <div class="api-input">
            <input 
              v-model="apiKey" 
              type="password" 
              placeholder="Enter your API key..."
              class="input-field"
            />
            <button @click="testApiKey" class="btn-test" :disabled="!apiKey || isTesting">
              {{ isTesting ? 'Testing...' : 'Test' }}
            </button>
          </div>
          
          <div v-if="keyStatus" :class="['key-status', keyStatus]">
            {{ keyStatus === 'valid' ? '✅ API key is valid!' : '❌ Invalid API key' }}
          </div>
        </div>
        
        <div class="step-actions">
          <button @click="skipStep" class="btn-secondary">Skip for now</button>
          <button @click="saveApiKey" class="btn-primary" :disabled="!apiKey">
            Continue
          </button>
        </div>
      </div>

      <!-- Step 4: Select Model -->
      <div v-if="currentStep === 4" class="step-panel">
        <h2>🤖 Select Default Model</h2>
        <p class="step-desc">Choose the AI model for your agents.</p>
        
        <div class="model-grid">
          <div 
            v-for="model in models" 
            :key="model.id"
            :class="['model-card', { selected: selectedModel === model.id }]"
            @click="selectedModel = model.id"
          >
            <div class="model-icon">{{ model.icon }}</div>
            <div class="model-name">{{ model.name }}</div>
            <div class="model-price">{{ model.price }}</div>
            <div v-if="model.recommended" class="model-badge">Recommended</div>
          </div>
        </div>
        
        <div class="step-actions">
          <button @click="nextStep" class="btn-primary">
            Continue
          </button>
        </div>
      </div>

      <!-- Step 5: Channel Setup (Optional) -->
      <div v-if="currentStep === 5" class="step-panel">
        <h2>📡 Connect Channel (Optional)</h2>
        <p class="step-desc">Connect messaging platforms to interact with your AI agents.</p>
        
        <div class="channel-grid">
          <div 
            v-for="channel in channels" 
            :key="channel.id"
            :class="['channel-card', { connected: channel.connected }]"
          >
            <div class="channel-logo">{{ channel.icon }}</div>
            <div class="channel-name">{{ channel.name }}</div>
            <button 
              @click="connectChannel(channel.id)" 
              class="btn-channel"
              :disabled="channel.connected"
            >
              {{ channel.connected ? 'Connected' : 'Connect' }}
            </button>
          </div>
        </div>
        
        <div class="step-actions">
          <button @click="skipStep" class="btn-secondary">Skip for now</button>
          <button @click="nextStep" class="btn-primary">
            Continue
          </button>
        </div>
      </div>

      <!-- Step 6: Start Gateway -->
      <div v-if="currentStep === 6" class="step-panel">
        <h2>🚀 启动 OpenClaw 服务</h2>
        <p class="step-desc">启动视控舱安全网关和 OpenClaw 核心引擎。</p>

        <div class="service-status-grid">
          <!-- Proxy Status -->
          <div class="service-card" :class="proxyStatus">
            <div class="service-header">
              <span class="service-icon">🛡️</span>
              <span class="service-title">视控舱安全网关</span>
            </div>
            <div class="service-port">Port: 18788</div>
            <div class="service-status-indicator">
              <span v-if="proxyStatus === 'pending'" class="status-pending">⏳ 等待启动</span>
              <span v-else-if="proxyStatus === 'starting'" class="status-starting">🔄 启动中...</span>
              <span v-else-if="proxyStatus === 'running'" class="status-running">✅ 运行中</span>
              <span v-else-if="proxyStatus === 'failed'" class="status-failed">❌ 失败</span>
            </div>
          </div>

          <!-- Gateway Status -->
          <div class="service-card" :class="gatewayStatus">
            <div class="service-header">
              <span class="service-icon">🦞</span>
              <span class="service-title">OpenClaw 核心引擎</span>
            </div>
            <div class="service-port">Port: 18789</div>
            <div class="service-status-indicator">
              <span v-if="gatewayStatus === 'pending'" class="status-pending">⏳ 等待启动</span>
              <span v-else-if="gatewayStatus === 'starting'" class="status-starting">🔄 启动中...</span>
              <span v-else-if="gatewayStatus === 'running'" class="status-running">✅ 运行中</span>
              <span v-else-if="gatewayStatus === 'failed'" class="status-failed">❌ 失败</span>
            </div>
          </div>
        </div>

        <!-- Startup progress message -->
        <div v-if="startupMessage" class="startup-message">
          {{ startupMessage }}
        </div>

        <div v-if="gatewayStarted" class="success-message">
          🎉 OpenClaw 服务已全部启动！正在跳转到主页...
        </div>

        <div class="step-actions">
          <button @click="startGateway" class="btn-primary btn-large" :disabled="isInstalling || gatewayStarted">
            {{ gatewayStarted ? '✅ 服务运行中' : '🚀 启动服务' }}
          </button>
        </div>
      </div>
    </div>

    <!-- Skip / Back buttons -->
    <div class="setup-footer">
      <button v-if="currentStep > 0" @click="prevStep" class="btn-back">
        ← Back
      </button>
      <div class="spacer"></div>
      <button v-if="currentStep < steps.length - 1 && currentStep > 0" @click="skipStep" class="btn-skip">
        Skip this step
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

const router = useRouter()

// Step definitions
const steps = [
  { id: 'env', label: 'Detection' },
  { id: 'node', label: 'Node.js' },
  { id: 'openclaw', label: 'OpenClaw' },
  { id: 'apikey', label: 'API Key' },
  { id: 'model', label: 'Model' },
  { id: 'channel', label: 'Channel' },
  { id: 'gateway', label: 'Gateway' },
]

const currentStep = ref(0)
const isChecking = ref(false)
const isInstalling = ref(false)
const isTesting = ref(false)
const useMirror = ref(false)
const gatewayStarted = ref(false)

// Gateway startup status
type ServiceStatus = 'pending' | 'starting' | 'running' | 'failed'
const proxyStatus = ref<ServiceStatus>('pending')
const gatewayStatus = ref<ServiceStatus>('pending')
const startupMessage = ref('')

// Environment check - use nested key paths for backend structure
const envItems = ref([
  { name: 'Node.js', key: 'node.installed', version_key: 'node.version', status: 'checking', version: '' },
  { name: 'npm', key: 'npm.installed', version_key: 'npm.version', status: 'checking', version: '' },
  { name: 'OpenClaw', key: 'openclaw.installed', version_key: 'openclaw.version', status: 'checking', version: '' },
  { name: 'Gateway', key: 'gateway.running', version_key: 'gateway.port', status: 'checking', version: '' },
])

// API configuration
const providers = [
  { id: 'anthropic', name: 'Anthropic' },
  { id: 'openai', name: 'OpenAI' },
  { id: 'deepseek', name: 'DeepSeek' },
]
const selectedProvider = ref('anthropic')
const apiKey = ref('')
const keyStatus = ref('')

// Models
const models = [
  { id: 'claude-3-5-sonnet-20241022', name: 'Claude 3.5 Sonnet', icon: '🧠', price: '$3/$15 per 1M tokens', recommended: true },
  { id: 'gpt-4o', name: 'GPT-4o', icon: '🤖', price: '$5/$15 per 1M tokens', recommended: false },
  { id: 'deepseek-chat', name: 'DeepSeek Chat', icon: '💬', price: '$0.14/$0.28 per 1M tokens', recommended: false },
]
const selectedModel = ref('claude-3-5-sonnet-20241022')

// Channels
const channels = ref([
  { id: 'telegram', name: 'Telegram', icon: '📱', connected: false },
  { id: 'discord', name: 'Discord', icon: '🎮', connected: false },
  { id: 'wechat', name: 'WeChat', icon: '💬', connected: false },
])

// Gateway
const gatewayPort = ref(18789)

// Install progress
const installProgress = ref({
  step: '',
  percent: 0,
  message: '',
  log_line: '',
})

// Listen for setup-progress events
onMounted(async () => {
  await listen('setup-progress', (event) => {
    installProgress.value = event.payload as any
  })
  
  // Start environment check
  await checkEnvironment()
})

// Helper to get nested property from object using dot notation
function getNestedValue(obj: any, path: string): any {
  return path.split('.').reduce((acc, key) => acc?.[key], obj)
}

async function checkEnvironment() {
  isChecking.value = true
  
  try {
    const env = await invoke<any>('check_environment')
    
    envItems.value = envItems.value.map(item => ({
      ...item,
      status: getNestedValue(env, item.key) ? 'ok' : 'missing',
      version: getNestedValue(env, item.version_key) || '',
    }))
    
    // Skip to appropriate step based on what's installed
    const nodeInstalled = getNestedValue(env, 'node.installed')
    const openclawInstalled = getNestedValue(env, 'openclaw.installed')
    const gatewayRunning = getNestedValue(env, 'gateway.running')
    
    if (nodeInstalled && openclawInstalled) {
      if (gatewayRunning) {
        // Everything is ready, go to dashboard
        router.push('/')
      } else {
        // Skip to gateway step
        currentStep.value = 6
      }
    } else if (nodeInstalled) {
      currentStep.value = 2 // OpenClaw installation
    }
  } catch (error) {
    console.error('Environment check failed:', error)
  } finally {
    isChecking.value = false
  }
}

function startSetup() {
  // Find first missing item
  const missingIndex = envItems.value.findIndex(item => item.status === 'missing')
  if (missingIndex === 0) {
    currentStep.value = 1 // Node.js
  } else if (missingIndex === 2) {
    currentStep.value = 2 // OpenClaw
  } else {
    currentStep.value = 3 // API Key
  }
}

async function installNode() {
  isInstalling.value = true
  installProgress.value = { step: 'install_node', percent: 0, message: 'Starting...', log_line: '' }
  
  try {
    await invoke('install_node')
    await checkEnvironment()
    nextStep()
  } catch (error) {
    console.error('Node.js installation failed:', error)
    installProgress.value.message = `Failed: ${error}`
  } finally {
    isInstalling.value = false
  }
}

async function installOpenClaw() {
  isInstalling.value = true
  installProgress.value = { step: 'install_openclaw', percent: 0, message: 'Starting...', log_line: '' }
  
  try {
    await invoke('install_openclaw', { useMirror: useMirror.value })
    await checkEnvironment()
    nextStep()
  } catch (error) {
    console.error('OpenClaw installation failed:', error)
    installProgress.value.message = `Failed: ${error}`
  } finally {
    isInstalling.value = false
  }
}

async function testApiKey() {
  isTesting.value = true
  keyStatus.value = ''
  
  try {
    const valid = await invoke<boolean>('test_api_key', {
      provider: selectedProvider.value,
      key: apiKey.value,
    })
    keyStatus.value = valid ? 'valid' : 'invalid'
  } catch (error) {
    keyStatus.value = 'invalid'
  } finally {
    isTesting.value = false
  }
}

async function saveApiKey() {
  try {
    await invoke('save_api_key', {
      provider: selectedProvider.value,
      key: apiKey.value,
    })
    nextStep()
  } catch (error) {
    console.error('Failed to save API key:', error)
  }
}

async function startGateway() {
  isInstalling.value = true
  proxyStatus.value = 'pending'
  gatewayStatus.value = 'pending'
  startupMessage.value = ''

  try {
    // Step 1: Configure OpenClaw proxy (hijack API Base URL to 18788)
    startupMessage.value = '正在配置视控舱安全网关...'
    proxyStatus.value = 'starting'
    
    await invoke('configure_openclaw_proxy', { proxyPort: 18788 })
    
    // Step 2: Start proxy on port 18788
    startupMessage.value = '正在启动视控舱安全网关 (Port 18788)...'
    await invoke('start_proxy', { port: 18788 })
    
    // Wait 1 second to confirm proxy is running
    await new Promise(resolve => setTimeout(resolve, 1000))
    proxyStatus.value = 'running'
    
    // Step 3: Start Gateway on port 18789
    startupMessage.value = '正在启动 OpenClaw 核心引擎 (Port 18789)...'
    gatewayStatus.value = 'starting'
    
    await invoke('start_gateway', { port: 18789 })
    gatewayStatus.value = 'running'
    
    gatewayStarted.value = true
    startupMessage.value = '所有服务已成功启动！'
    
    // Wait a moment then go to dashboard
    setTimeout(() => {
      router.push('/')
    }, 2000)
  } catch (error) {
    console.error('Failed to start services:', error)
    
    // Update failed status based on which step failed
    if (proxyStatus.value === 'starting') {
      proxyStatus.value = 'failed'
      startupMessage.value = `视控舱安全网关启动失败: ${error}`
    } else if (gatewayStatus.value === 'starting') {
      gatewayStatus.value = 'failed'
      startupMessage.value = `OpenClaw 核心引擎启动失败: ${error}`
    }
  } finally {
    isInstalling.value = false
  }
}

function nextStep() {
  if (currentStep.value < steps.length - 1) {
    currentStep.value++
  }
}

function prevStep() {
  if (currentStep.value > 0) {
    currentStep.value--
  }
}

function skipStep() {
  nextStep()
}

async function connectChannel(channelId: string) {
  // TODO: Implement channel connection
  console.log('Connecting channel:', channelId)
}
</script>

<style scoped>
.setup-wizard {
  min-height: 100vh;
  background: linear-gradient(135deg, #0a0a0f 0%, #1a1a2e 100%);
  color: #e0e0e0;
  display: flex;
  flex-direction: column;
  padding: 2rem;
}

.setup-progress {
  display: flex;
  justify-content: center;
  gap: 1rem;
  margin-bottom: 3rem;
}

.step-indicator {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.5rem;
  opacity: 0.5;
  transition: all 0.3s ease;
}

.step-indicator.active {
  opacity: 1;
}

.step-indicator.completed {
  opacity: 0.8;
}

.step-number {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  background: #2a2a3e;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: bold;
}

.step-indicator.active .step-number {
  background: #3b82f6;
  box-shadow: 0 0 20px rgba(59, 130, 246, 0.5);
}

.step-indicator.completed .step-number {
  background: #22c55e;
}

.step-label {
  font-size: 0.75rem;
  color: #888;
}

.setup-content {
  flex: 1;
  display: flex;
  justify-content: center;
  align-items: center;
}

.step-panel {
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 16px;
  padding: 3rem;
  max-width: 600px;
  width: 100%;
}

.step-panel h2 {
  font-size: 1.5rem;
  margin-bottom: 0.5rem;
}

.step-desc {
  color: #888;
  margin-bottom: 2rem;
}

.env-list {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.env-item {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 1rem;
  background: rgba(255, 255, 255, 0.05);
  border-radius: 8px;
}

.env-icon {
  font-size: 1.5rem;
}

.env-name {
  font-weight: 500;
}

.env-version {
  font-size: 0.75rem;
  color: #888;
}

.install-progress {
  margin: 1.5rem 0;
}

.progress-bar {
  height: 8px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 4px;
  overflow: hidden;
  margin-bottom: 1rem;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, #3b82f6, #8b5cf6);
  transition: width 0.3s ease;
}

.progress-message {
  font-size: 0.875rem;
  color: #888;
}

.log-output {
  margin-top: 0.5rem;
  padding: 0.5rem;
  background: rgba(0, 0, 0, 0.3);
  border-radius: 4px;
  font-family: monospace;
  font-size: 0.75rem;
  color: #666;
  overflow-x: auto;
}

.mirror-option {
  margin-bottom: 1.5rem;
}

.mirror-option label {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  cursor: pointer;
}

.provider-tabs {
  display: flex;
  gap: 0.5rem;
  margin-bottom: 1.5rem;
}

.provider-tab {
  padding: 0.5rem 1rem;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  color: #888;
  cursor: pointer;
  transition: all 0.2s;
}

.provider-tab.active {
  background: #3b82f6;
  color: white;
  border-color: #3b82f6;
}

.api-input {
  display: flex;
  gap: 0.5rem;
}

.input-field {
  flex: 1;
  padding: 0.75rem;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  color: white;
  font-size: 1rem;
}

.input-field:focus {
  outline: none;
  border-color: #3b82f6;
}

.btn-test {
  padding: 0.75rem 1.5rem;
  background: rgba(255, 255, 255, 0.1);
  border: none;
  border-radius: 8px;
  color: white;
  cursor: pointer;
}

.key-status {
  margin-top: 0.5rem;
  padding: 0.5rem;
  border-radius: 4px;
  font-size: 0.875rem;
}

.key-status.valid {
  color: #22c55e;
}

.key-status.invalid {
  color: #ef4444;
}

.model-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 1rem;
}

.model-card {
  padding: 1.5rem;
  background: rgba(255, 255, 255, 0.05);
  border: 2px solid rgba(255, 255, 255, 0.1);
  border-radius: 12px;
  text-align: center;
  cursor: pointer;
  transition: all 0.2s;
  position: relative;
}

.model-card:hover {
  border-color: rgba(59, 130, 246, 0.5);
}

.model-card.selected {
  border-color: #3b82f6;
  background: rgba(59, 130, 246, 0.1);
}

.model-icon {
  font-size: 2rem;
  margin-bottom: 0.5rem;
}

.model-name {
  font-weight: 500;
  margin-bottom: 0.25rem;
}

.model-price {
  font-size: 0.75rem;
  color: #888;
}

.model-badge {
  position: absolute;
  top: -8px;
  right: -8px;
  background: #22c55e;
  color: white;
  padding: 0.25rem 0.5rem;
  border-radius: 4px;
  font-size: 0.625rem;
}

.channel-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 1rem;
}

.channel-card {
  padding: 1.5rem;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 12px;
  text-align: center;
}

.channel-logo {
  font-size: 2rem;
  margin-bottom: 0.5rem;
}

.channel-name {
  font-weight: 500;
  margin-bottom: 1rem;
}

.btn-channel {
  padding: 0.5rem 1rem;
  background: rgba(59, 130, 246, 0.2);
  border: 1px solid #3b82f6;
  border-radius: 6px;
  color: #3b82f6;
  cursor: pointer;
}

.channel-card.connected .btn-channel {
  background: rgba(34, 197, 94, 0.2);
  border-color: #22c55e;
  color: #22c55e;
}

.gateway-config {
  margin-bottom: 1.5rem;
}

.port-input {
  width: 100px;
}

.service-status-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 1.5rem;
  margin-bottom: 1.5rem;
}

.service-card {
  padding: 1.5rem;
  background: rgba(255, 255, 255, 0.05);
  border: 2px solid rgba(255, 255, 255, 0.1);
  border-radius: 12px;
  text-align: center;
  transition: all 0.3s ease;
}

.service-card.pending {
  border-color: rgba(255, 255, 255, 0.1);
}

.service-card.starting {
  border-color: rgba(59, 130, 246, 0.5);
  background: rgba(59, 130, 246, 0.1);
}

.service-card.running {
  border-color: rgba(34, 197, 94, 0.5);
  background: rgba(34, 197, 94, 0.1);
}

.service-card.failed {
  border-color: rgba(239, 68, 68, 0.5);
  background: rgba(239, 68, 68, 0.1);
}

.service-header {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  margin-bottom: 0.5rem;
}

.service-icon {
  font-size: 1.5rem;
}

.service-title {
  font-weight: 500;
  font-size: 1rem;
}

.service-port {
  font-size: 0.875rem;
  color: #888;
  margin-bottom: 0.75rem;
}

.service-status-indicator {
  font-size: 0.875rem;
  font-weight: 500;
}

.status-pending {
  color: #888;
}

.status-starting {
  color: #3b82f6;
}

.status-running {
  color: #22c55e;
}

.status-failed {
  color: #ef4444;
}

.startup-message {
  padding: 0.75rem 1rem;
  background: rgba(59, 130, 246, 0.1);
  border: 1px solid rgba(59, 130, 246, 0.3);
  border-radius: 8px;
  color: #60a5fa;
  text-align: center;
  margin: 1rem 0;
  font-size: 0.875rem;
}

.success-message {
  padding: 1rem;
  background: rgba(34, 197, 94, 0.1);
  border: 1px solid rgba(34, 197, 94, 0.3);
  border-radius: 8px;
  color: #22c55e;
  text-align: center;
  margin: 1rem 0;
}

.step-actions {
  display: flex;
  gap: 1rem;
  margin-top: 2rem;
}

.btn-primary {
  flex: 1;
  padding: 0.875rem 1.5rem;
  background: #3b82f6;
  border: none;
  border-radius: 8px;
  color: white;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-primary:hover:not(:disabled) {
  background: #2563eb;
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-primary.btn-large {
  padding: 1.25rem 2rem;
  font-size: 1.125rem;
}

.btn-secondary {
  padding: 0.875rem 1.5rem;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  color: #888;
  cursor: pointer;
}

.setup-footer {
  display: flex;
  justify-content: space-between;
  margin-top: 2rem;
}

.btn-back, .btn-skip {
  padding: 0.5rem 1rem;
  background: transparent;
  border: none;
  color: #888;
  cursor: pointer;
}

.btn-back:hover, .btn-skip:hover {
  color: white;
}

.spacer {
  flex: 1;
}
</style>