<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, UnlistenFn } from '@tauri-apps/api/event'

const emit = defineEmits(['complete', 'next', 'back'])

const installing = ref(false)
const progress = ref(0)
const message = ref('')
const logs = ref<string[]>([])
const error = ref('')
const completed = ref(false)

let unlisten: UnlistenFn | null = null

async function startInstall() {
  installing.value = true
  error.value = ''
  progress.value = 0
  message.value = '开始安装...'
  logs.value = []

  try {
    await invoke('install_node', { useMirror: false })
    completed.value = true
    progress.value = 100
    message.value = '安装完成!'
  } catch (e) {
    error.value = `安装失败: ${e}`
  } finally {
    installing.value = false
  }
}

function proceed() {
  emit('complete', { nodeInstalled: true })
  emit('next')
}

onMounted(async () => {
  // Check if already installed
  try {
    const status = await invoke('get_env_status')
    const node = (status as any).node
    if (node?.installed) {
      completed.value = true
      progress.value = 100
      message.value = 'Node.js 已安装'
    }
  } catch (e) {
    console.error('Failed to check node status:', e)
  }

  // Listen for progress events
  unlisten = await listen<any>('setup-progress', (event) => {
    if (event.payload.step === 'install_node') {
      progress.value = event.payload.percent
      message.value = event.payload.message
      if (event.payload.log_line) {
        logs.value.push(event.payload.log_line)
        // Keep only last 50 lines
        if (logs.value.length > 50) {
          logs.value = logs.value.slice(-50)
        }
      }
      if (event.payload.percent >= 100) {
        completed.value = true
      }
    }
  })
})

onUnmounted(() => {
  if (unlisten) unlisten()
})
</script>

<template>
  <div class="node-installer">
    <h2>📦 安装 Node.js</h2>
    <p class="subtitle">在您的系统上安装 Node.js 运行时</p>

    <div v-if="completed" class="success-card">
      <div class="success-icon">✅</div>
      <h3>Node.js 已安装</h3>
      <p>版本检查通过，可以继续下一步</p>
    </div>

    <div v-else class="install-panel">
      <div class="info-box">
        <h4>ℹ️ 安装说明</h4>
        <ul>
          <li><strong>macOS:</strong> 使用 Homebrew 安装 (推荐)</li>
          <li><strong>Windows:</strong> 使用 winget 或手动下载安装包</li>
          <li><strong>Linux:</strong> 使用包管理器 (apt/dnf)</li>
        </ul>
      </div>

      <div class="progress-section">
        <div class="progress-header">
          <span>{{ message }}</span>
          <span>{{ progress }}%</span>
        </div>
        <div class="progress-bar">
          <div class="progress-fill" :style="{ width: `${progress}%` }"></div>
        </div>
      </div>

      <div v-if="logs.length > 0" class="log-panel">
        <div class="log-header">安装日志:</div>
        <div class="log-content">
          <div v-for="(log, i) in logs" :key="i" class="log-line">{{ log }}</div>
        </div>
      </div>

      <div v-if="error" class="error-box">
        {{ error }}
      </div>

      <div class="actions">
        <button 
          class="btn btn-primary" 
          :disabled="installing"
          @click="startInstall"
        >
          {{ installing ? '安装中...' : '开始安装 Node.js' }}
        </button>
      </div>
    </div>

    <div class="footer-actions">
      <button class="btn btn-secondary" @click="$emit('back')">
        ← 上一步
      </button>
      <button 
        class="btn btn-primary" 
        :disabled="!completed"
        @click="proceed"
      >
        继续 →
      </button>
    </div>
  </div>
</template>

<style scoped>
.node-installer {
  max-width: 700px;
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

.success-card {
  text-align: center;
  padding: 48px;
  background: var(--bg-card);
  border-radius: 12px;
  border: 2px solid var(--green);
  margin-bottom: 24px;
}

.success-icon {
  font-size: 48px;
  margin-bottom: 16px;
}

.info-box {
  background: var(--bg-card);
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 24px;
}

.info-box h4 {
  margin-bottom: 8px;
}

.info-box ul {
  margin: 0;
  padding-left: 20px;
  color: var(--text-secondary);
}

.info-box li {
  margin-bottom: 4px;
}

.progress-section {
  margin-bottom: 16px;
}

.progress-header {
  display: flex;
  justify-content: space-between;
  margin-bottom: 8px;
  font-size: 14px;
}

.progress-bar {
  height: 8px;
  background: var(--bg-card);
  border-radius: 4px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, var(--cyan), var(--amber));
  transition: width 0.3s ease;
}

.log-panel {
  background: var(--bg-card);
  border-radius: 8px;
  margin-bottom: 16px;
  max-height: 200px;
  overflow: hidden;
}

.log-header {
  padding: 8px 12px;
  background: var(--bg-base);
  font-size: 12px;
  color: var(--text-secondary);
}

.log-content {
  padding: 8px 12px;
  font-family: monospace;
  font-size: 12px;
  max-height: 150px;
  overflow-y: auto;
}

.log-line {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  color: var(--text-secondary);
}

.error-box {
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid var(--red);
  border-radius: 8px;
  padding: 12px;
  color: var(--red);
  margin-bottom: 16px;
}

.actions {
  display: flex;
  justify-content: center;
}

.footer-actions {
  display: flex;
  justify-content: space-between;
  margin-top: 32px;
}

.btn {
  padding: 12px 24px;
  border-radius: 8px;
  font-weight: 600;
  cursor: pointer;
  border: none;
  transition: all 0.2s;
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
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
