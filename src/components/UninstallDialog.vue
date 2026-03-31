<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const emit = defineEmits(['uninstall-complete'])

const showDialog = ref(false)
const step = ref(1) // 1: options, 2: confirm, 3: progress, 4: complete
const confirmText = ref('')
const uninstalling = ref(false)
const progress = ref(0)
const logs = ref<string[]>([])

const options = ref({
  stopGateway: true,
  removeCli: true,
  removeConfig: true,
  removeNode: false
})

function openDialog() {
  showDialog.value = true
  step.value = 1
  confirmText.value = ''
}

function proceedToConfirm() {
  if (options.value.removeConfig || options.value.removeCli) {
    step.value = 2
  } else {
    startUninstall()
  }
}

async function startUninstall() {
  step.value = 3
  uninstalling.value = true
  progress.value = 10
  
  try {
    await invoke('uninstall_openclaw', {
      scope: {
        stop_gateway: options.value.stopGateway,
        remove_cli: options.value.removeCli,
        remove_config: options.value.removeConfig,
        remove_node: options.value.removeNode
      }
    })
    
    progress.value = 100
    step.value = 4
    
    emit('uninstall-complete')
  } catch (e) {
    logs.value.push(`错误: ${e}`)
  } finally {
    uninstalling.value = false
  }
}

function closeDialog() {
  showDialog.value = false
}
</script>

<template>
  <div class="danger-zone">
    <h3>⚠️ 危险操作</h3>
    <p>卸载 OpenClaw 及相关数据。此操作不可恢复。</p>
    
    <button class="btn btn-danger" @click="openDialog">
      一键卸载 OpenClaw
    </button>

    <!-- Dialog -->
    <div v-if="showDialog" class="modal-overlay" @click.self="closeDialog">
      <div class="modal">
        <!-- Step 1: Options -->
        <div v-if="step === 1">
          <h3>🗑 卸载选项</h3>
          
          <div class="options">
            <label class="option">
              <input type="checkbox" v-model="options.stopGateway">
              <span>停止并移除 Gateway 服务</span>
            </label>
            
            <label class="option">
              <input type="checkbox" v-model="options.removeCli">
              <span>移除 OpenClaw CLI</span>
            </label>
            
            <label class="option">
              <input type="checkbox" v-model="options.removeConfig">
              <span>删除配置和数据 (~/.openclaw)</span>
            </label>
            
            <label class="option warning">
              <input type="checkbox" v-model="options.removeNode">
              <span>同时移除 Node.js (不推荐)</span>
            </label>
          </div>

          <div class="modal-actions">
            <button class="btn btn-secondary" @click="closeDialog">取消</button>
            <button class="btn btn-primary" @click="proceedToConfirm">继续</button>
          </div>
        </div>

        <!-- Step 2: Confirm -->
        <div v-if="step === 2">
          <h3>⚠️ 确认卸载</h3>
          
          <div class="warning-box">
            <p>您即将卸载 OpenClaw，这将会：</p>
            <ul>
              <li v-if="options.stopGateway">停止 Gateway 服务</li>
              <li v-if="options.removeCli">删除 openclaw 命令</li>
              <li v-if="options.removeConfig">删除所有配置和数据</li>
              <li v-if="options.removeNode">删除 Node.js 运行时</li>
            </ul>
          </div>

          <div class="confirm-input">
            <label>请输入 <code>UNINSTALL</code> 确认:</label>
            <input type="text" v-model="confirmText" placeholder="UNINSTALL">
          </div>

          <div class="modal-actions">
            <button class="btn btn-secondary" @click="step = 1">返回</button>
            <button 
              class="btn btn-danger" 
              :disabled="confirmText !== 'UNINSTALL'"
              @click="startUninstall"
            >
              确认卸载
            </button>
          </div>
        </div>

        <!-- Step 3: Progress -->
        <div v-if="step === 3">
          <h3>🔄 正在卸载...</h3>
          
          <div class="progress-section">
            <div class="progress-bar">
              <div class="progress-fill" :style="{ width: `${progress}%` }"></div>
            </div>
            <span>{{ progress }}%</span>
          </div>

          <div v-if="logs.length > 0" class="log-panel">
            <div v-for="(log, i) in logs" :key="i">{{ log }}</div>
          </div>
        </div>

        <!-- Step 4: Complete -->
        <div v-if="step === 4">
          <div class="success-icon">✅</div>
          <h3>卸载完成</h3>
          <p>OpenClaw 已从您的系统中移除。</p>
          
          <div class="modal-actions">
            <button class="btn btn-primary" @click="closeDialog">关闭</button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.danger-zone {
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid var(--red);
  border-radius: 12px;
  padding: 20px;
  margin-top: 32px;
}

.danger-zone h3 {
  color: var(--red);
  margin: 0 0 8px 0;
}

.danger-zone p {
  color: var(--text-secondary);
  margin: 0 0 16px 0;
}

/* Modal */
.modal-overlay {
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

.modal {
  background: var(--bg-base);
  border-radius: 16px;
  padding: 24px;
  width: 450px;
  max-width: 90%;
}

.modal h3 {
  margin: 0 0 20px 0;
}

.options {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-bottom: 24px;
}

.option {
  display: flex;
  align-items: center;
  gap: 12px;
  cursor: pointer;
}

.option input {
  width: 18px;
  height: 18px;
}

.option.warning {
  color: var(--amber);
}

.warning-box {
  background: rgba(239, 68, 68, 0.1);
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 20px;
}

.warning-box ul {
  margin: 12px 0 0 0;
  padding-left: 20px;
}

.confirm-input {
  margin-bottom: 24px;
}

.confirm-input label {
  display: block;
  margin-bottom: 8px;
}

.confirm-input code {
  background: var(--bg-card);
  padding: 2px 6px;
  border-radius: 4px;
}

.confirm-input input {
  width: 100%;
  padding: 12px;
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  color: var(--text-primary);
  font-size: 16px;
  text-align: center;
}

.progress-section {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-bottom: 16px;
}

.progress-bar {
  flex: 1;
  height: 8px;
  background: var(--bg-card);
  border-radius: 4px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: var(--red);
  transition: width 0.3s;
}

.log-panel {
  background: var(--bg-card);
  border-radius: 8px;
  padding: 12px;
  max-height: 150px;
  overflow-y: auto;
  font-size: 12px;
  font-family: monospace;
}

.success-icon {
  font-size: 48px;
  text-align: center;
  margin-bottom: 16px;
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

.btn {
  padding: 10px 20px;
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

.btn-danger {
  background: var(--red);
  color: white;
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
