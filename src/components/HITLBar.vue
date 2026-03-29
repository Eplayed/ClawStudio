<script setup lang="ts">
/**
 * HITLBar.vue - Human-In-The-Loop 审批栏
 * 三种状态：inactive（权限选择）/ alert（待审批）/ correction（纠正输入）
 */
import { ref, computed, watch, onUnmounted } from 'vue'
import { useSettingsStore } from '@/stores/settings'

export type PermLevel = 'browse' | 'standard' | 'auto'
export type HITLState = 'inactive' | 'alert' | 'correction'

const settingsStore = useSettingsStore()

// Props
const props = defineProps<{
  agentName?: string
  actionDesc?: string
  actionTag?: string
}>()

const emit = defineEmits<{
  'perm-change': [level: PermLevel]
  approve: []
  reject: [correction: string]
  takeover: []
  timeout: []
}>()

const state = ref<HITLState>('inactive')

// Simulated pending request (in real app, driven by event stream)
const pendingAgent = ref('AGENT-01')
const pendingAction = ref('删除文件 /invoice/2024-03.pdf')
const pendingTag = ref('DELETE')

// Timer
const countdown = ref(30)
let timerInterval: number | undefined

function startTimer() {
  countdown.value = 30
  timerInterval = window.setInterval(() => {
    countdown.value--
    if (countdown.value <= 0) {
      clearInterval(timerInterval!)
      emit('timeout')
      state.value = 'inactive'
    }
  }, 1000)
}

function stopTimer() {
  if (timerInterval) {
    clearInterval(timerInterval)
    timerInterval = undefined
  }
}

// Simulate receiving a HITL request (for demo / dev)
watch(
  () => props.actionDesc,
  (desc) => {
    if (desc) {
      pendingAction.value = desc
      pendingTag.value = props.actionTag || 'ACTION'
      state.value = 'alert'
      startTimer()
    }
  }
)

// Demo: trigger an alert for dev
function triggerDemoAlert() {
  pendingAgent.value = props.agentName || 'AGENT-01'
  pendingAction.value = '点击「删除」按钮'
  pendingTag.value = 'DELETE'
  state.value = 'alert'
  startTimer()
}

// Actions
function approve() {
  stopTimer()
  state.value = 'inactive'
  emit('approve')
}

function rejectWithCorrection(correction: string) {
  stopTimer()
  state.value = 'inactive'
  emit('reject', correction)
}

function takeover() {
  stopTimer()
  state.value = 'inactive'
  emit('takeover')
}

function cancelCorrection() {
  stopTimer()
  state.value = 'alert'
  startTimer()
}

// Permission level switch
function setPermLevel(level: PermLevel) {
  settingsStore.permLevel = level
  emit('perm-change', level)
  showToast(level)
}

const correctionText = ref('')

function submitCorrection() {
  if (correctionText.value.trim()) {
    rejectWithCorrection(correctionText.value.trim())
    correctionText.value = ''
  }
}

// Toast
const toastVisible = ref(false)
const toastMsg = ref('')
let toastTimer: number | undefined

function showToast(perm: PermLevel) {
  toastMsg.value = perm === 'browse' ? '已切换为「仅浏览」' : perm === 'standard' ? '已切换为「标准」' : '已切换为「完全自主」'
  toastVisible.value = true
  if (toastTimer) clearTimeout(toastTimer)
  toastTimer = window.setTimeout(() => {
    toastVisible.value = false
  }, 2500)
}

onUnmounted(stopTimer)

const permLabels: Record<PermLevel, string> = {
  browse: '仅浏览',
  standard: '标准',
  auto: '完全自主',
}
</script>

<template>
  <div class="hitl-bar">
    <!-- INACTIVE: Permission level selector -->
    <div v-if="state === 'inactive'" class="hitl-inactive">
      <div class="perm-level">
        <span class="shield-icon">🛡</span>
        <span>授权级别：</span>
        <div class="perm-select">
          <button
            class="perm-option"
            :class="{ 'active-browse': settingsStore.permLevel === 'browse' }"
            @click="setPermLevel('browse')"
          >
            仅浏览
          </button>
          <button
            class="perm-option"
            :class="{ 'active-standard': settingsStore.permLevel === 'standard' }"
            @click="setPermLevel('standard')"
          >
            标准
          </button>
          <button
            class="perm-option"
            :class="{ 'active-auto': settingsStore.permLevel === 'auto' }"
            @click="setPermLevel('auto')"
          >
            完全自主
          </button>
        </div>
      </div>
      <div class="hitl-status-row">
        <span class="hitl-badge">
          HITL {{ settingsStore.hitlEnabled ? '已启用' : '已禁用' }}
        </span>
        <button
          class="demo-btn"
          @click="triggerDemoAlert"
          title="模拟 HITL 审批请求（开发调试用）"
        >
          模拟测试
        </button>
      </div>
    </div>

    <!-- ALERT: Pending approval -->
    <div v-if="state === 'alert'" class="hitl-alert show">
      <div class="hitl-alert-content">
        <div class="warn-icon">⚠️</div>
        <div class="alert-msg">
          <strong class="agent-tag">{{ pendingAgent }}</strong>
          请求授权执行：
          <strong class="action-tag">{{ pendingAction }}</strong>
        </div>
        <div class="hitl-actions">
          <button class="hitl-btn approve" @click="approve">✅ 批准</button>
          <button class="hitl-btn reject" @click="state = 'correction'; correctionText = ''">❌ 拒绝+纠正</button>
          <button class="hitl-btn takeover" @click="takeover">🎮 接管</button>
        </div>
        <div class="hitl-timer">
          ⏱ {{ countdown }}s
        </div>
      </div>
    </div>

    <!-- CORRECTION: Text input -->
    <div v-if="state === 'correction'" class="hitl-correct-box show">
      <div class="correct-header">
        <span class="correct-label">✏️ 拒绝并纠正</span>
        <button class="cancel-btn" @click="cancelCorrection">取消</button>
      </div>
      <div class="hitl-correct-row">
        <input
          v-model="correctionText"
          class="hitl-correct-input"
          placeholder="输入正确的操作指令，例如：「不要删除，将文件移至回收站」"
          @keyup.enter="submitCorrection"
        />
        <button class="hitl-correct-send" @click="submitCorrection">
          发送纠正
        </button>
      </div>
    </div>

    <!-- Toast notification -->
    <div
      class="perm-toast"
      :class="[settingsStore.permLevel, { show: toastVisible }]"
    >
      {{ toastMsg }}
    </div>
  </div>
</template>

<style scoped>
.hitl-bar {
  border-top: 1px solid var(--border);
  background: var(--bg-base);
  flex-shrink: 0;
}

/* Inactive */
.hitl-inactive {
  padding: 10px 20px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  color: var(--text-dim);
  font-size: 12px;
}

.perm-level {
  display: flex;
  align-items: center;
  gap: 8px;
}

.shield-icon {
  font-size: 14px;
}

.hitl-status-row {
  display: flex;
  align-items: center;
  gap: 10px;
}

.hitl-badge {
  font-size: 11px;
  padding: 3px 10px;
  border-radius: 10px;
  background: var(--green-glow);
  color: var(--green);
  border: 1px solid rgba(34, 197, 94, 0.2);
}

.demo-btn {
  font-size: 10px;
  padding: 3px 10px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border);
  background: var(--bg-elevated);
  color: var(--text-dim);
  cursor: pointer;
  transition: all 0.2s;
  font-family: var(--font-ui);
}

.demo-btn:hover {
  border-color: var(--cyan-dim);
  color: var(--cyan);
}

/* Alert */
.hitl-alert {
  padding: 12px 20px;
  background: linear-gradient(90deg, rgba(239, 68, 68, 0.08), rgba(239, 68, 68, 0.03));
  border-top: 2px solid var(--red);
  animation: hitl-flash 2s infinite;
}

@keyframes hitl-flash {
  0%, 100% { border-top-color: var(--red); }
  50%       { border-top-color: rgba(239, 68, 68, 0.4); }
}

.hitl-alert-content {
  display: flex;
  align-items: center;
  gap: 12px;
}

.warn-icon {
  font-size: 22px;
  animation: shake 0.6s ease-in-out infinite alternate;
  flex-shrink: 0;
}

@keyframes shake {
  0%  { transform: rotate(-3deg); }
  100%{ transform: rotate(3deg); }
}

.alert-msg {
  font-size: 13px;
  color: var(--text-primary);
  line-height: 1.4;
  flex: 1;
}

.agent-tag {
  color: var(--cyan);
  font-weight: 600;
}

.action-tag {
  color: var(--red);
  font-weight: 600;
  background: var(--red-glow);
  padding: 1px 6px;
  border-radius: 3px;
}

.hitl-actions {
  display: flex;
  gap: 8px;
  margin-left: 8px;
}

.hitl-btn {
  padding: 8px 20px;
  border-radius: var(--radius-sm);
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  border: none;
  transition: all 0.2s;
  font-family: var(--font-ui);
}

.hitl-btn.approve {
  background: var(--green);
  color: #fff;
  box-shadow: 0 0 15px var(--green-glow);
}

.hitl-btn.approve:hover { background: #16a34a; }

.hitl-btn.reject {
  background: var(--red);
  color: #fff;
  box-shadow: 0 0 15px var(--red-glow);
}

.hitl-btn.reject:hover { background: #dc2626; }

.hitl-btn.takeover {
  background: var(--bg-elevated);
  color: var(--text-primary);
  border: 1px solid var(--border-light);
}

.hitl-btn.takeover:hover {
  border-color: var(--cyan-dim);
  color: var(--cyan);
}

.hitl-timer {
  font-family: var(--font-mono);
  font-size: 12px;
  color: var(--red);
  display: flex;
  align-items: center;
  gap: 5px;
  margin-left: 10px;
  animation: blink-soft 1s infinite;
  min-width: 45px;
}

@keyframes blink-soft {
  0%, 100% { opacity: 1; }
  50%       { opacity: 0.5; }
}

/* Correction box */
.hitl-correct-box {
  padding: 12px 20px;
  background: var(--bg-card);
  border-top: 1px solid var(--border);
  animation: modalFadeIn 0.2s ease;
}

@keyframes modalFadeIn {
  from { opacity: 0; transform: translateY(-4px); }
  to   { opacity: 1; transform: translateY(0); }
}

.correct-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
}

.correct-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--amber);
}

.cancel-btn {
  font-size: 11px;
  padding: 3px 10px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border);
  background: transparent;
  color: var(--text-dim);
  cursor: pointer;
  transition: all 0.2s;
  font-family: var(--font-ui);
}

.cancel-btn:hover {
  border-color: var(--border-light);
  color: var(--text-secondary);
}

.hitl-correct-row {
  display: flex;
  gap: 8px;
  align-items: flex-start;
}

.hitl-correct-input {
  flex: 1;
  padding: 9px 12px;
  background: var(--bg-base);
  border: 1px solid var(--amber-dim);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-family: var(--font-ui);
  font-size: 12px;
  outline: none;
  transition: border-color 0.2s;
}

.hitl-correct-input:focus {
  border-color: var(--amber);
  box-shadow: 0 0 0 2px var(--amber-glow);
}

.hitl-correct-input::placeholder {
  color: var(--text-dim);
}

.hitl-correct-send {
  padding: 9px 18px;
  border-radius: var(--radius-sm);
  border: none;
  background: var(--amber);
  color: #000;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  font-family: var(--font-ui);
  white-space: nowrap;
  transition: background 0.2s;
}

.hitl-correct-send:hover { background: #e09020; }

/* Permission selector */
.perm-select {
  display: flex;
  gap: 6px;
  align-items: center;
}

.perm-option {
  font-size: 10px;
  padding: 3px 10px;
  border-radius: 12px;
  border: 1px solid var(--border);
  cursor: pointer;
  color: var(--text-dim);
  transition: all 0.2s;
  background: transparent;
  font-family: var(--font-ui);
  font-weight: 500;
}

.perm-option:hover {
  border-color: var(--border-light);
  color: var(--text-secondary);
}

.perm-option.active-browse   { border-color: var(--green-dim); color: var(--green); background: var(--green-glow); }
.perm-option.active-standard { border-color: var(--amber-dim); color: var(--amber); background: var(--amber-glow); }
.perm-option.active-auto     { border-color: var(--red-dim); color: var(--red); background: var(--red-glow); }

/* Toast */
.perm-toast {
  position: fixed;
  top: 20px;
  right: 20px;
  padding: 12px 20px;
  border-radius: var(--radius-sm);
  font-size: 12px;
  font-weight: 600;
  z-index: 600;
  transform: translateX(120%);
  transition: transform 0.35s cubic-bezier(0.4, 0, 0.2, 1);
  box-shadow: 0 8px 30px rgba(0, 0, 0, 0.4);
  pointer-events: none;
}

.perm-toast.show { transform: translateX(0); }
.perm-toast.browse   { background: var(--green); color: #000; }
.perm-toast.standard { background: var(--amber); color: #000; }
.perm-toast.auto     { background: var(--red); color: #fff; }
</style>
