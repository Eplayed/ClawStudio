<script setup lang="ts">
/**
 * VncConnectModal.vue - VNC 连接弹窗
 */
import { ref, computed } from 'vue'

const props = defineProps<{
  visible: boolean
  currentVncUrl?: string
  currentVncPort?: number
}>()

const emit = defineEmits<{
  close: []
  connect: [url: string]
  disconnect: []
}>()

const vncUrl = ref('')
const vncPort = ref(6080)
const connecting = ref(false)
const error = ref('')

// 初始化
const initUrl = computed(() => {
  if (props.currentVncUrl) return props.currentVncUrl
  if (props.currentVncPort) return `ws://localhost:${props.currentVncPort}`
  return 'ws://localhost:6080'
})

function handleConnect() {
  error.value = ''
  
  if (!vncUrl.value.trim()) {
    error.value = '请输入 VNC 地址'
    return
  }

  connecting.value = true
  
  // 验证 WebSocket URL
  try {
    const url = vncUrl.value.trim()
    if (!url.startsWith('ws://') && !url.startsWith('wss://')) {
      error.value = 'VNC 地址必须以 ws:// 或 wss:// 开头'
      connecting.value = false
      return
    }
    
    emit('connect', url)
    connecting.value = false
  } catch (e) {
    error.value = '无效的 VNC 地址'
    connecting.value = false
  }
}

function handleDisconnect() {
  emit('disconnect')
}

// 预设模板
const presets = [
  { label: '本地 6080', url: 'ws://localhost:6080' },
  { label: '本地 5901', url: 'ws://localhost:5901' },
  { label: 'Docker 6080', url: 'ws://127.0.0.1:6080' },
]

function selectPreset(url: string) {
  vncUrl.value = url
}
</script>

<template>
  <div v-if="visible" class="modal-overlay show" @click.self="emit('close')">
    <div class="modal">
      <!-- Header -->
      <div class="modal-header">
        <h2>🖥 连接 VNC</h2>
        <button class="modal-close" @click="emit('close')">✕</button>
      </div>

      <!-- Body -->
      <div class="modal-body">
        <!-- Quick presets -->
        <div class="form-group">
          <label class="form-label">快速连接</label>
          <div class="preset-btns">
            <button
              v-for="p in presets"
              :key="p.url"
              class="preset-btn"
              @click="selectPreset(p.url)"
            >
              {{ p.label }}
            </button>
          </div>
        </div>

        <!-- Custom URL -->
        <div class="form-group">
          <label class="form-label">VNC WebSocket 地址</label>
          <input
            v-model="vncUrl"
            class="form-input"
            placeholder="ws://localhost:6080"
          />
          <div class="form-hint">
            支持 ws:// (非加密) 或 wss:// (加密) 协议
          </div>
        </div>

        <!-- Error message -->
        <div v-if="error" class="form-error">
          {{ error }}
        </div>

        <!-- Info -->
        <div class="vnc-info">
          <h4>💡 连接说明</h4>
          <ul>
            <li>确保沙盒容器已启动并暴露 VNC 端口</li>
            <li>Docker 镜像 <code>dorowu/ubuntu-desktop-lxde-vnc</code> 使用端口 80 → 映射到 6080</li>
            <li>如需密码认证，请在连接时输入</li>
          </ul>
        </div>
      </div>

      <!-- Footer -->
      <div class="modal-footer">
        <button class="modal-btn cancel" @click="emit('close')">取消</button>
        <button
          class="modal-btn primary"
          :disabled="connecting"
          @click="handleConnect"
        >
          {{ connecting ? '连接中…' : '连接' }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.7);
  backdrop-filter: blur(4px);
  z-index: 500;
  display: flex;
  align-items: center;
  justify-content: center;
  animation: modalFadeIn 0.25s ease;
}

.modal-overlay.show { display: flex; }

@keyframes modalFadeIn {
  from { opacity: 0; }
  to   { opacity: 1; }
}

.modal {
  background: var(--bg-card);
  border: 1px solid var(--border-light);
  border-radius: 12px;
  width: 420px;
  max-height: 85vh;
  overflow-y: auto;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.6), 0 0 40px var(--cyan-glow);
  animation: modalSlideIn 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

@keyframes modalSlideIn {
  from { opacity: 0; transform: translateY(20px) scale(0.97); }
  to   { opacity: 1; transform: translateY(0) scale(1); }
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 18px 22px;
  border-bottom: 1px solid var(--border);
}

.modal-header h2 {
  font-size: 15px;
  font-weight: 600;
  display: flex;
  align-items: center;
  gap: 8px;
}

.modal-close {
  width: 28px;
  height: 28px;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: transparent;
  color: var(--text-dim);
  cursor: pointer;
  font-size: 14px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
}

.modal-close:hover {
  border-color: var(--red-dim);
  color: var(--red);
  background: var(--red-glow);
}

.modal-body {
  padding: 20px 22px;
}

.modal-footer {
  padding: 14px 22px;
  border-top: 1px solid var(--border);
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}

.modal-btn {
  padding: 8px 20px;
  border-radius: var(--radius-sm);
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  font-family: var(--font-ui);
  border: 1px solid var(--border);
  transition: all 0.2s;
}

.modal-btn.cancel {
  background: var(--bg-elevated);
  color: var(--text-secondary);
}

.modal-btn.cancel:hover {
  border-color: var(--border-light);
  color: var(--text-primary);
}

.modal-btn.primary {
  background: linear-gradient(135deg, var(--cyan), #0891b2);
  color: #fff;
  border: none;
  box-shadow: 0 0 15px var(--cyan-glow);
}

.modal-btn.primary:hover {
  box-shadow: 0 0 25px var(--cyan-glow-strong);
}

.modal-btn.primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.form-group {
  margin-bottom: 16px;
}

.form-group:last-child {
  margin-bottom: 0;
}

.form-label {
  display: block;
  font-size: 11px;
  font-weight: 600;
  color: var(--text-secondary);
  margin-bottom: 6px;
  letter-spacing: 0.3px;
}

.form-hint {
  font-size: 10px;
  color: var(--text-dim);
  margin-top: 4px;
}

.form-input {
  width: 100%;
  padding: 9px 12px;
  background: var(--bg-base);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: 12px;
  transition: border-color 0.2s;
  outline: none;
}

.form-input:focus {
  border-color: var(--cyan-dim);
  box-shadow: 0 0 0 2px var(--cyan-glow);
}

.form-input::placeholder {
  color: var(--text-dim);
}

.form-error {
  margin-top: 12px;
  padding: 10px 14px;
  background: var(--red-glow);
  border: 1px solid rgba(239, 68, 68, 0.3);
  border-radius: var(--radius-sm);
  font-size: 11px;
  color: var(--red);
}

.preset-btns {
  display: flex;
  gap: 8px;
}

.preset-btn {
  padding: 6px 12px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border);
  background: var(--bg-base);
  color: var(--text-secondary);
  font-size: 11px;
  cursor: pointer;
  transition: all 0.2s;
  font-family: var(--font-ui);
}

.preset-btn:hover {
  border-color: var(--cyan-dim);
  color: var(--cyan);
}

.vnc-info {
  margin-top: 16px;
  padding: 12px;
  background: var(--bg-base);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
}

.vnc-info h4 {
  font-size: 12px;
  font-weight: 600;
  margin-bottom: 8px;
  color: var(--text-secondary);
}

.vnc-info ul {
  margin: 0;
  padding-left: 16px;
  font-size: 11px;
  color: var(--text-dim);
  line-height: 1.6;
}

.vnc-info code {
  background: rgba(0, 0, 0, 0.3);
  padding: 1px 6px;
  border-radius: 3px;
  font-family: var(--font-mono);
  font-size: 10px;
  color: var(--cyan);
}
</style>
