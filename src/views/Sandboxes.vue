<script setup lang="ts">
/**
 * Sandboxes.vue - 沙盒环境管理页面
 * D5: 本机/Docker 运行模式切换
 * D6: CPU/Memory 实时 mini-charts
 * D7: 快捷配置编辑器
 */
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useSettingsStore } from '@/stores/settings'
import SandboxCard from '@/components/SandboxCard.vue'
import DockerGuide from '@/components/DockerGuide.vue'

interface DockerStatus {
  available: boolean
  version: string
  containers_total: number
  containers_running: number
  message: string
}

interface SandboxInfo {
  id: string
  name: string
  image: string
  status: string
  vnc_port: number
  created_at: string
}

interface SandboxStats {
  cpu_percent: number
  memory_used_mb: number
  memory_limit_mb: number
}

const settingsStore = useSettingsStore()

const dockerStatus = ref<DockerStatus | null>(null)
const sandboxes = ref<SandboxInfo[]>([])
const sandboxStats = ref<Record<string, SandboxStats>>({})
const loading = ref(false)
const showGuide = ref(false)
const creating = ref(false)

// D5: Run mode
const runMode = computed({
  get: () => settingsStore.runMode,
  set: (val) => {
    settingsStore.runMode = val
    settingsStore.saveAllSettings()
  }
})

// D6: Stats polling
let statsInterval: ReturnType<typeof setInterval> | undefined

async function pollStats() {
  for (const sb of sandboxes.value) {
    if (sb.status === 'running') {
      try {
        const stats = await invoke<SandboxStats>('get_sandbox_stats', { container_id: sb.id })
        sandboxStats.value[sb.id] = stats
      } catch (e) {
        // Container may have stopped
      }
    }
  }
}

async function checkDocker() {
  loading.value = true
  try {
    dockerStatus.value = await invoke<DockerStatus>('check_docker')
    showGuide.value = !dockerStatus.value!.available
    if (dockerStatus.value?.available) {
      await loadSandboxes()
    }
  } catch (e) {
    dockerStatus.value = {
      available: false,
      version: '',
      containers_total: 0,
      containers_running: 0,
      message: String(e),
    }
    showGuide.value = true
  }
  loading.value = false
}

async function loadSandboxes() {
  try {
    sandboxes.value = await invoke<SandboxInfo[]>('list_sandboxes')
  } catch (e) {
    console.warn('list_sandboxes failed:', e)
    sandboxes.value = []
  }
}

async function createSandbox() {
  creating.value = true
  try {
    const name = `ubuntu-${Date.now().toString(36)}`
    await invoke('create_sandbox', { name })
    await loadSandboxes()
    await checkDocker()
  } catch (e) {
    console.error('Failed to create sandbox:', e)
  }
  creating.value = false
}

async function destroySandbox(id: string) {
  try {
    await invoke('destroy_sandbox', { container_id: id })
    await loadSandboxes()
    await checkDocker()
  } catch (e) {
    console.error('Failed to destroy sandbox:', e)
  }
}

async function stopSandbox(id: string) {
  console.log('Stop sandbox:', id)
}

async function startSandbox(id: string) {
  console.log('Start sandbox:', id)
}

function openVNC(sandbox: { id: string; name: string; vnc_port: number }) {
  const url = `vnc://localhost:${sandbox.vnc_port}`
  window.open(url, '_blank')
}

onMounted(() => {
  checkDocker()
  // D6: Start polling stats every 5 seconds
  statsInterval = setInterval(pollStats, 5000)
})

onUnmounted(() => {
  if (statsInterval) clearInterval(statsInterval)
})
</script>

<template>
  <div class="page">
    <!-- Top bar -->
    <header class="topbar">
      <div class="page-title">📦 沙盒环境 Sandboxes</div>
      <div class="topbar-actions">
        <button class="btn-refresh" @click="checkDocker" :disabled="loading">
          {{ loading ? '⏳' : '🔄' }}
        </button>
      </div>
    </header>

    <!-- Content -->
    <div class="sandbox-content">
      <!-- D5: Run Mode Toggle -->
      <div class="run-mode-toggle">
        <div class="toggle-label">运行模式 Run Mode</div>
        <div class="toggle-buttons">
          <button
            class="toggle-btn"
            :class="{ active: runMode === 'local' }"
            @click="runMode = 'local'"
          >
            🖥 本机运行 (Local)
          </button>
          <button
            class="toggle-btn"
            :class="{ active: runMode === 'docker' }"
            @click="runMode = 'docker'"
          >
            🐳 Docker 沙盒 (Docker)
          </button>
        </div>
        <p v-if="runMode === 'local'" class="mode-warning">
          ⚠️ 本机模式下 Agent 直接操作你的系统，请确保 HITL 审批已开启
        </p>
      </div>

      <!-- Local Mode: Environment Info -->
      <div v-if="runMode === 'local'" class="local-mode-panel">
        <h3>🖥 本机运行环境</h3>
        <div class="env-stats">
          <div class="env-stat">
            <div class="env-label">运行环境</div>
            <div class="env-value">本机</div>
          </div>
          <div class="env-stat">
            <div class="env-label">Node.js 版本</div>
            <div class="env-value">v20.11.0</div>
          </div>
          <div class="env-stat">
            <div class="env-label">Python 版本</div>
            <div class="env-value">3.12.1</div>
          </div>
          <div class="env-stat">
            <div class="env-label">HITL 状态</div>
            <div class="env-value" :style="{ color: settingsStore.hitlEnabled ? 'var(--green)' : 'var(--red)' }">
              {{ settingsStore.hitlEnabled ? '已启用' : '已禁用' }}
            </div>
          </div>
        </div>
      </div>

      <!-- Docker Guide (shown if Docker unavailable) -->
      <DockerGuide
        v-if="showGuide && runMode === 'docker'"
        @retry="checkDocker"
        @close="showGuide = false"
      />

      <!-- Docker Mode Content -->
      <template v-if="runMode === 'docker'">
        <!-- Docker status panel -->
        <div v-if="dockerStatus" class="docker-panel">
          <h3>
            🐳 Docker 状态
            <span class="docker-badge" :class="dockerStatus.available ? 'ok' : 'error'">
              {{ dockerStatus.available ? '已连接' : '未连接' }}
            </span>
          </h3>
          <div class="docker-stats" v-if="dockerStatus.available">
            <div class="docker-stat">
              <div class="ds-val" style="color: var(--text-primary)">{{ dockerStatus.version }}</div>
              <div class="ds-label">Docker 版本</div>
            </div>
            <div class="docker-stat">
              <div class="ds-val" style="color: var(--text-primary)">{{ dockerStatus.containers_total }}</div>
              <div class="ds-label">容器总数</div>
            </div>
            <div class="docker-stat">
              <div class="ds-val" style="color: var(--green)">{{ dockerStatus.containers_running }}</div>
              <div class="ds-label">运行中</div>
            </div>
            <div class="docker-stat">
              <div class="ds-val" style="color: var(--text-secondary)">{{ dockerStatus.containers_total - dockerStatus.containers_running }}</div>
              <div class="ds-label">已停止</div>
            </div>
          </div>
        </div>

        <!-- Sandbox grid -->
        <div class="sandbox-top-actions" v-if="dockerStatus?.available">
          <h2 style="font-size:14px;font-weight:600">🏗 沙盒实例</h2>
          <button
            class="btn-create-sandbox"
            @click="createSandbox"
            :disabled="creating"
          >
            {{ creating ? '⏳ 创建中…' : '+ 创建新沙盒' }}
          </button>
        </div>

        <div class="sandbox-grid" v-if="dockerStatus?.available">
          <SandboxCard
            v-for="sb in sandboxes"
            :key="sb.id"
            :sandbox="{
              id: sb.id,
              name: sb.name,
              image: sb.image,
              status: sb.status,
              vnc_port: sb.vnc_port
            }"
            :stats="sandboxStats[sb.id]"
            @connect-vnc="openVNC"
            @destroy="destroySandbox"
            @start="startSandbox"
            @stop="stopSandbox"
          />

          <div v-if="sandboxes.length === 0" class="sandbox-empty">
            <p>暂无沙盒实例</p>
            <p>点击上方「创建新沙盒」开始</p>
          </div>
        </div>
      </template>

      <!-- D7: Quick Config -->
      <div class="quick-config-section">
        <h3>🔧 快捷配置 Quick Config</h3>
        <div class="config-grid">
          <!-- Model Selection -->
          <div class="config-block">
            <div class="config-label">当前模型</div>
            <select v-model="settingsStore.defaultModel" @change="settingsStore.saveAllSettings()" class="config-select">
              <option value="claude-3-5-sonnet-20241022">Claude 3.5 Sonnet</option>
              <option value="claude-3-opus-20240229">Claude 3 Opus</option>
              <option value="gpt-4o">GPT-4o</option>
            </select>
          </div>

          <!-- API Key Status -->
          <div class="config-block">
            <div class="config-label">API Key 状态</div>
            <div class="key-status">
              <div class="key-item">
                <span class="key-badge" :class="{ valid: settingsStore.claudeKeyValid }">
                  {{ settingsStore.claudeKeyValid ? '✅' : '❌' }}
                </span>
                Claude Key
              </div>
              <div class="key-item">
                <span class="key-badge" :class="{ valid: settingsStore.openaiKeyValid }">
                  {{ settingsStore.openaiKeyValid ? '✅' : '❌' }}
                </span>
                OpenAI Key
              </div>
            </div>
            <a href="/settings" class="config-link">前往设置 →</a>
          </div>

          <!-- HITL Toggle -->
          <div class="config-block">
            <div class="config-label">HITL 审批</div>
            <div class="hitl-toggle">
              <label class="toggle-switch">
                <input type="checkbox" v-model="settingsStore.hitlEnabled" @change="settingsStore.saveAllSettings()">
                <span class="slider"></span>
              </label>
              <span class="hitl-status">{{ settingsStore.hitlEnabled ? '已启用' : '已禁用' }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.page {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.topbar {
  height: 52px;
  min-height: 52px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 24px;
  border-bottom: 1px solid var(--border);
  background: var(--bg-base);
  flex-shrink: 0;
}

.page-title {
  font-size: 15px;
  font-weight: 600;
}

.topbar-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.btn-refresh {
  width: 32px;
  height: 32px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border);
  background: var(--bg-card);
  color: var(--text-secondary);
  cursor: pointer;
  font-size: 14px;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
}

.btn-refresh:hover {
  border-color: var(--cyan-dim);
  color: var(--cyan);
}

.sandbox-content {
  flex: 1;
  overflow-y: auto;
  padding: 20px 24px;
}

/* D5: Run Mode Toggle */
.run-mode-toggle {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 18px;
  margin-bottom: 20px;
}

.toggle-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary);
  margin-bottom: 12px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.toggle-buttons {
  display: flex;
  gap: 10px;
  margin-bottom: 12px;
}

.toggle-btn {
  flex: 1;
  padding: 10px 16px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border);
  background: var(--bg-base);
  color: var(--text-secondary);
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.toggle-btn:hover {
  border-color: var(--cyan-dim);
  color: var(--cyan);
}

.toggle-btn.active {
  background: var(--cyan-glow);
  border-color: var(--cyan-dim);
  color: var(--cyan);
  box-shadow: 0 0 15px var(--cyan-glow);
}

.mode-warning {
  margin: 0;
  padding: 10px 12px;
  background: var(--amber-glow);
  border: 1px solid rgba(240, 160, 48, 0.2);
  border-radius: var(--radius-sm);
  font-size: 11px;
  color: var(--amber);
}

/* Local Mode Panel */
.local-mode-panel {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 18px;
  margin-bottom: 20px;
}

.local-mode-panel h3 {
  font-size: 13px;
  font-weight: 600;
  margin-bottom: 14px;
}

.env-stats {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 12px;
}

.env-stat {
  background: var(--bg-base);
  border-radius: var(--radius-sm);
  padding: 12px;
  text-align: center;
}

.env-label {
  font-size: 10px;
  color: var(--text-dim);
  margin-bottom: 6px;
}

.env-value {
  font-size: 14px;
  font-weight: 600;
  font-family: var(--font-mono);
  color: var(--text-primary);
}

/* Docker panel */
.docker-panel {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 18px;
  margin-bottom: 20px;
}

.docker-panel h3 {
  font-size: 13px;
  font-weight: 600;
  margin-bottom: 14px;
  display: flex;
  align-items: center;
  gap: 8px;
}

.docker-badge {
  font-size: 10px;
  padding: 2px 8px;
  border-radius: 10px;
  font-weight: 500;
}

.docker-badge.ok { background: var(--green-glow); color: var(--green); }
.docker-badge.error { background: var(--red-glow); color: var(--red); }

.docker-stats {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 12px;
}

.docker-stat {
  background: var(--bg-base);
  border-radius: var(--radius-sm);
  padding: 12px;
  text-align: center;
}

.docker-stat .ds-val {
  font-size: 20px;
  font-weight: 700;
  font-family: var(--font-mono);
  margin-bottom: 4px;
}

.docker-stat .ds-label {
  font-size: 10px;
  color: var(--text-dim);
}

/* Sandbox actions */
.sandbox-top-actions {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.btn-create-sandbox {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 9px 20px;
  background: linear-gradient(135deg, var(--cyan), #0891b2);
  border: none;
  border-radius: var(--radius-sm);
  color: #fff;
  font-family: var(--font-ui);
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  box-shadow: 0 0 20px var(--cyan-glow);
  transition: all 0.25s;
}

.btn-create-sandbox:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 0 30px var(--cyan-glow-strong);
}

.btn-create-sandbox:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

/* Sandbox grid */
.sandbox-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: 14px;
  margin-bottom: 24px;
}

.sandbox-empty {
  grid-column: 1 / -1;
  text-align: center;
  padding: 48px;
  color: var(--text-dim);
  font-size: 13px;
  line-height: 2;
}

/* D7: Quick Config */
.quick-config-section {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 20px;
}

.quick-config-section h3 {
  font-size: 14px;
  font-weight: 600;
  margin-bottom: 16px;
}

.config-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
}

.config-block {
  background: var(--bg-base);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  padding: 14px;
}

.config-label {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: 10px;
}

.config-select {
  width: 100%;
  padding: 8px 10px;
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-size: 12px;
  font-family: var(--font-ui);
  cursor: pointer;
}

.config-select:focus {
  outline: none;
  border-color: var(--cyan-dim);
  box-shadow: 0 0 0 2px var(--cyan-glow);
}

.key-status {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 10px;
}

.key-item {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
}

.key-badge {
  font-size: 14px;
}

.key-badge.valid {
  color: var(--green);
}

.config-link {
  font-size: 11px;
  color: var(--cyan);
  text-decoration: none;
  cursor: pointer;
}

.config-link:hover {
  text-decoration: underline;
}

.hitl-toggle {
  display: flex;
  align-items: center;
  gap: 10px;
}

.toggle-switch {
  position: relative;
  display: inline-block;
  width: 40px;
  height: 22px;
}

.toggle-switch input {
  opacity: 0;
  width: 0;
  height: 0;
}

.slider {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: var(--border);
  transition: 0.3s;
  border-radius: 22px;
}

.slider:before {
  position: absolute;
  content: '';
  height: 18px;
  width: 18px;
  left: 2px;
  bottom: 2px;
  background-color: white;
  transition: 0.3s;
  border-radius: 50%;
}

input:checked + .slider {
  background-color: var(--green);
}

input:checked + .slider:before {
  transform: translateX(18px);
}

.hitl-status {
  font-size: 12px;
  color: var(--text-secondary);
}

/* Responsive */
@media (max-width: 900px) {
  .config-grid { grid-template-columns: repeat(2, 1fr); }
  .env-stats { grid-template-columns: repeat(2, 1fr); }
  .docker-stats { grid-template-columns: repeat(2, 1fr); }
}

@media (max-width: 600px) {
  .config-grid { grid-template-columns: 1fr; }
  .env-stats { grid-template-columns: 1fr; }
  .docker-stats { grid-template-columns: 1fr; }
}
</style>
