<script setup lang="ts">
/**
 * CreateAgentModal.vue - 创建 Agent 弹窗
 * 表单校验：Computer Use 开启时必须选沙盒
 */
import { ref, computed } from 'vue'
import { useAgentStore, type Agent } from '@/stores/agents'
import { invoke } from '@tauri-apps/api/core'

const props = defineProps<{
  sandboxes?: Array<{ id: string; name: string; vnc_port: number }>
}>()

const emit = defineEmits<{
  close: []
  created: [agent: Agent]
}>()

const agentStore = useAgentStore()

// Form state
const name = ref('')
const avatar = ref('🤖')
const systemPrompt = ref('')
const computerUse = ref(false)
const networkAccess = ref(true)
const sandboxId = ref('')
const budgetLimit = ref(1.0)
const tokenLimit = ref(100000)
const loading = ref(false)
const errorMsg = ref('')

// Avatar options
const avatarOptions = ['🤖', '🔍', '📊', '💬', '🎯', '⚡', '🌐', '📝', '📁', '🔧']

const showCUWarning = computed(() => {
  return computerUse.value && !sandboxId.value
})

const isFormValid = computed(() => {
  return name.value.trim().length > 0 && !showCUWarning.value
})

function selectAvatar(a: string) {
  avatar.value = a
}

function toggleCU() {
  computerUse.value = !computerUse.value
  if (computerUse.value && !sandboxId.value) {
    // Auto-select first sandbox if available
    if (props.sandboxes && props.sandboxes.length > 0) {
      sandboxId.value = props.sandboxes[0].id
    }
  } else if (!computerUse.value) {
    sandboxId.value = ''
  }
}

async function submit() {
  errorMsg.value = ''

  if (!name.value.trim()) {
    errorMsg.value = '请输入 Agent 名称'
    return
  }

  if (showCUWarning.value) {
    errorMsg.value = '启用 Computer Use 时，必须选择一个沙盒环境'
    return
  }

  loading.value = true

  try {
    const id = `agent-${Date.now().toString(36)}`
    const agent: Agent = {
      id,
      name: name.value.trim(),
      avatar: avatar.value,
      systemPrompt: systemPrompt.value.trim(),
      computerUse: computerUse.value,
      networkAccess: networkAccess.value,
      sandboxId: sandboxId.value || null,
      fileWhitelist: '',
      budgetLimit: budgetLimit.value,
      tokenLimit: tokenLimit.value,
      status: 'queued',
      currentCost: 0,
      tokensUsed: 0,
      elapsedSec: 0,
      progress: 0,
    }

    // Save to DB
    try {
      await invoke('save_agent_config', {
        agent: {
          id: agent.id,
          name: agent.name,
          avatar: agent.avatar,
          system_prompt: agent.systemPrompt,
          computer_use: agent.computerUse,
          network_access: agent.networkAccess,
          sandbox_id: agent.sandboxId,
          file_whitelist: agent.fileWhitelist,
          budget_limit: agent.budgetLimit,
          token_limit: agent.tokenLimit,
        },
      })
    } catch (e) {
      console.warn('DB save failed, continuing anyway:', e)
    }

    agentStore.addAgent(agent)
    emit('created', agent)
    emit('close')
  } catch (e) {
    errorMsg.value = String(e)
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div class="modal-overlay show" @click.self="emit('close')">
    <div class="modal">
      <!-- Header -->
      <div class="modal-header">
        <h2>🤖 创建新特工</h2>
        <button class="modal-close" @click="emit('close')">✕</button>
      </div>

      <!-- Body -->
      <div class="modal-body">
        <!-- Agent name -->
        <div class="form-group">
          <label class="form-label">名称 Name *</label>
          <input
            v-model="name"
            class="form-input"
            placeholder="例如：整理发票、监控竞品、客服回复"
            maxlength="60"
          />
        </div>

        <!-- Avatar picker -->
        <div class="form-group">
          <label class="form-label">头像</label>
          <div class="avatar-picker">
            <button
              v-for="a in avatarOptions"
              :key="a"
              class="avatar-opt"
              :class="{ selected: avatar === a }"
              @click="selectAvatar(a)"
            >
              {{ a }}
            </button>
          </div>
        </div>

        <!-- System prompt -->
        <div class="form-group">
          <label class="form-label">系统提示词 System Prompt</label>
          <textarea
            v-model="systemPrompt"
            class="form-textarea"
            placeholder="描述 Agent 的角色、能力范围和行为规范…"
            rows="4"
          ></textarea>
        </div>

        <!-- Computer Use toggle -->
        <div class="form-group">
          <div class="toggle-row">
            <div class="toggle-info">
              <label class="form-label" style="margin:0">Computer Use</label>
              <span class="form-hint">启用桌面操作能力（需要沙盒环境）</span>
            </div>
            <button
              class="toggle-track"
              :class="{ on: computerUse }"
              @click="toggleCU"
              type="button"
            >
              <div class="toggle-knob"></div>
            </button>
          </div>

          <!-- CU warning -->
          <div class="cu-warning" :class="{ show: showCUWarning }">
            ⚠️ Computer Use 已启用，请从下方选择一个沙盒环境，否则无法保存
          </div>
        </div>

        <!-- Sandbox selector (visible when CU is on) -->
        <div class="form-group" v-if="computerUse">
          <label class="form-label">沙盒环境 Sandbox *</label>
          <select v-model="sandboxId" class="form-select">
            <option value="">— 选择沙盒 —</option>
            <option
              v-for="sb in (sandboxes || [])"
              :key="sb.id"
              :value="sb.id"
            >
              {{ sb.name }} (端口 {{ sb.vnc_port }})
            </option>
          </select>
        </div>

        <!-- Network access -->
        <div class="form-group">
          <div class="toggle-row">
            <div class="toggle-info">
              <label class="form-label" style="margin:0">网络访问</label>
              <span class="form-hint">允许访问外部网络资源</span>
            </div>
            <button
              class="toggle-track"
              :class="{ on: networkAccess }"
              @click="networkAccess = !networkAccess"
              type="button"
            >
              <div class="toggle-knob"></div>
            </button>
          </div>
        </div>

        <!-- Budget & token limit -->
        <div class="form-row">
          <div class="form-group">
            <label class="form-label">预算上限 ($)</label>
            <input
              v-model.number="budgetLimit"
              type="number"
              class="form-input"
              min="0.01"
              max="1000"
              step="0.1"
            />
          </div>
          <div class="form-group">
            <label class="form-label">Token 限额</label>
            <input
              v-model.number="tokenLimit"
              type="number"
              class="form-input"
              min="1000"
              max="1000000"
              step="1000"
            />
          </div>
        </div>

        <!-- Error -->
        <div class="form-error" v-if="errorMsg">
          {{ errorMsg }}
        </div>
      </div>

      <!-- Footer -->
      <div class="modal-footer">
        <button class="modal-btn cancel" @click="emit('close')">取消</button>
        <button
          class="modal-btn primary"
          :disabled="!isFormValid || loading"
          @click="submit"
        >
          {{ loading ? '创建中…' : '创建 Agent' }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* Modal overlay already defined globally */
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

@keyframes modalFadeIn {
  from { opacity: 0; }
  to   { opacity: 1; }
}

.modal {
  background: var(--bg-card);
  border: 1px solid var(--border-light);
  border-radius: 12px;
  width: 560px;
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
  position: sticky;
  top: 0;
  background: var(--bg-card);
  z-index: 1;
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

/* Form styles */
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
}

.form-input,
.form-textarea,
.form-select {
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

.form-input:focus,
.form-textarea:focus,
.form-select:focus {
  border-color: var(--cyan-dim);
  box-shadow: 0 0 0 2px var(--cyan-glow);
}

.form-input::placeholder,
.form-textarea::placeholder {
  color: var(--text-dim);
}

.form-textarea {
  font-family: var(--font-ui);
  resize: vertical;
  min-height: 80px;
}

.form-select {
  font-family: var(--font-ui);
  appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='10' height='6'%3E%3Cpath d='M0 0l5 6 5-6z' fill='%234a5568'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 12px center;
}

.form-row {
  display: flex;
  gap: 12px;
}

.form-row .form-group {
  flex: 1;
}

/* Toggle */
.toggle-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
}

.toggle-info {
  flex: 1;
}

.toggle-track {
  width: 44px;
  height: 24px;
  background: var(--border);
  border-radius: 12px;
  position: relative;
  cursor: pointer;
  transition: background 0.3s;
  border: none;
  flex-shrink: 0;
}

.toggle-track.on {
  background: var(--cyan-dim);
}

.toggle-track .toggle-knob {
  width: 18px;
  height: 18px;
  background: #fff;
  border-radius: 50%;
  position: absolute;
  top: 3px;
  left: 3px;
  transition: left 0.3s;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.3);
}

.toggle-track.on .toggle-knob {
  left: 23px;
}

/* CU Warning */
.cu-warning {
  display: none;
  margin-top: 8px;
  padding: 10px 14px;
  background: var(--amber-glow);
  border: 1px solid rgba(240, 160, 48, 0.3);
  border-radius: var(--radius-sm);
  font-size: 11px;
  color: var(--amber);
  line-height: 1.5;
}

.cu-warning.show {
  display: block;
}

/* Avatar picker */
.avatar-picker {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.avatar-opt {
  width: 36px;
  height: 36px;
  border-radius: 8px;
  border: 2px solid var(--border);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 18px;
  cursor: pointer;
  transition: all 0.2s;
  background: var(--bg-base);
}

.avatar-opt:hover {
  border-color: var(--border-light);
}

.avatar-opt.selected {
  border-color: var(--cyan);
  box-shadow: 0 0 10px var(--cyan-glow);
}

/* Form error */
.form-error {
  margin-top: 12px;
  padding: 10px 14px;
  background: var(--red-glow);
  border: 1px solid rgba(239, 68, 68, 0.3);
  border-radius: var(--radius-sm);
  font-size: 11px;
  color: var(--red);
}
</style>
