<template>
  <div class="monitor-panel">
    <div class="monitor-header">
      <div class="header-left">
        <span class="pulse-dot" :class="{ active: isListening }"></span>
        <h3>LLM Monitor</h3>
        <span class="event-count">{{ store.monitorEvents.length }} events</span>
      </div>
      <div class="header-actions">
        <button @click="clearEvents" class="btn-clear">Clear</button>
        <button @click="toggleListening" class="btn-toggle" :class="{ active: isListening }">
          {{ isListening ? 'Pause' : 'Resume' }}
        </button>
      </div>
    </div>

    <div class="event-list" ref="eventListRef">
      <div v-if="store.monitorEvents.length === 0" class="empty-state">
        <div class="empty-icon">📡</div>
        <p>Waiting for Proxy events...</p>
        <p class="hint">触发一次模型调用后会出现 Token/Thinking/Action 等事件</p>
      </div>

      <div
        v-for="(event, index) in store.monitorEvents"
        :key="index"
        class="event-item"
        :class="event.type"
      >
        <div class="event-header">
          <span class="event-type">{{ getEventIcon(event.type) }}</span>
          <span class="event-time">{{ formatTime(event.timestamp) }}</span>
          <span class="event-badge" :class="event.type">{{ getEventLabel(event.type) }}</span>
        </div>

        <div class="event-body">
          <div v-if="event.type === 'llm_request'" class="request-info">
            <div class="info-row">
              <span class="label">Model:</span>
              <span class="value model">{{ event.model }}</span>
            </div>
            <div class="info-row">
              <span class="label">URL:</span>
              <span class="value url">{{ event.url }}</span>
            </div>
            <div class="info-row">
              <span class="label">Messages:</span>
              <span class="value">{{ event.messageCount }}</span>
            </div>
            <div class="info-row">
              <span class="label">Stream:</span>
              <span class="value">{{ event.stream ? 'Yes' : 'No' }}</span>
            </div>
          </div>

          <div v-else-if="event.type === 'llm_response'" class="response-info">
            <div class="info-row">
              <span class="label">Status:</span>
              <span class="value" :class="getStatusClass(event.status)">{{ event.status }}</span>
            </div>
            <div class="info-row">
              <span class="label">Duration:</span>
              <span class="value">{{ event.duration }}ms</span>
            </div>
            <div class="info-row">
              <span class="label">Model:</span>
              <span class="value model">{{ event.model }}</span>
            </div>
            <div v-if="event.usage" class="info-row">
              <span class="label">Tokens:</span>
              <span class="value">
                {{ event.usage.prompt_tokens || 0 }} in /
                {{ event.usage.completion_tokens || 0 }} out
              </span>
            </div>
            <div v-if="event.finishReason" class="info-row">
              <span class="label">Finish:</span>
              <span class="value">{{ event.finishReason }}</span>
            </div>
          </div>

          <div v-else-if="event.type === 'llm_error'" class="error-info">
            <div class="info-row">
              <span class="label">URL:</span>
              <span class="value url">{{ event.url }}</span>
            </div>
            <div class="error-message">{{ event.error }}</div>
          </div>
          
          <div v-else-if="event.type === 'proxy_token_usage'" class="response-info">
            <div class="info-row">
              <span class="label">Model:</span>
              <span class="value model">{{ event.model }}</span>
            </div>
            <div class="info-row">
              <span class="label">Tokens:</span>
              <span class="value">{{ event.input }} in / {{ event.output }} out</span>
            </div>
            <div class="info-row">
              <span class="label">Cost:</span>
              <span class="value success">${{ event.cost.toFixed(6) }}</span>
            </div>
          </div>

          <div v-else-if="event.type === 'proxy_thinking'" class="request-info">
            <div class="info-row">
              <span class="label">Step:</span>
              <span class="value">{{ event.step }}</span>
            </div>
            <div class="error-message">{{ event.text }}</div>
          </div>

          <div v-else-if="event.type === 'proxy_action'" class="request-info">
            <div class="info-row">
              <span class="label">Tool:</span>
              <span class="value model">{{ event.tool }}</span>
            </div>
            <div class="info-row">
              <span class="label">Step:</span>
              <span class="value">{{ event.step }}</span>
            </div>
            <div class="error-message">{{ JSON.stringify(event.params, null, 2) }}</div>
          </div>

          <div v-else-if="event.type === 'proxy_hitl_request'" class="error-info">
            <div class="info-row">
              <span class="label">HITL:</span>
              <span class="value model">{{ event.tool }}</span>
            </div>
            <div class="error-message">{{ JSON.stringify(event.params, null, 2) }}</div>
          </div>

          <div v-else-if="event.type === 'proxy_circuit_breaker'" class="error-info">
            <div class="error-message">{{ event.reason }} ({{ event.current_cost.toFixed(2) }} / {{ event.limit.toFixed(2) }})</div>
          </div>

          <div v-else-if="event.type === 'proxy_error'" class="error-info">
            <div class="error-message">{{ event.message }}</div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, nextTick, watch } from 'vue'
import { useProxyStore } from '@/stores/proxy'

const store = useProxyStore()
const eventListRef = ref<HTMLElement | null>(null)
const isListening = ref(true)

onMounted(async () => {
  // 初始化 store（如果还没初始化）
  if (store.monitorEvents.length === 0) {
    await store.init()
  }
})

watch(
  () => store.monitorEvents.length,
  async () => {
    if (!isListening.value) return
    await nextTick()
    if (eventListRef.value) {
      eventListRef.value.scrollTop = eventListRef.value.scrollHeight
    }
  }
)

function clearEvents() {
  store.monitorEvents = []
}

function toggleListening() {
  isListening.value = !isListening.value
}

function formatTime(timestamp: string): string {
  try {
    const date = new Date(timestamp)
    return date.toLocaleTimeString('en-US', { hour12: false })
  } catch {
    return timestamp
  }
}

function getEventIcon(type: string): string {
  switch (type) {
    case 'llm_request': return '📤'
    case 'llm_response': return '📥'
    case 'llm_error': return '❌'
    case 'proxy_token_usage': return '💰'
    case 'proxy_thinking': return '🧠'
    case 'proxy_action': return '🧰'
    case 'proxy_hitl_request': return '🛑'
    case 'proxy_circuit_breaker': return '⛔'
    case 'proxy_error': return '⚠️'
    default: return '📝'
  }
}

function getEventLabel(type: string): string {
  switch (type) {
    case 'llm_request': return 'REQUEST'
    case 'llm_response': return 'RESPONSE'
    case 'llm_error': return 'ERROR'
    case 'proxy_token_usage': return 'TOKEN'
    case 'proxy_thinking': return 'THINK'
    case 'proxy_action': return 'ACTION'
    case 'proxy_hitl_request': return 'HITL'
    case 'proxy_circuit_breaker': return 'CIRCUIT'
    case 'proxy_error': return 'PROXY_ERR'
    default: return type
  }
}

function getStatusClass(status: number): string {
  if (status >= 200 && status < 300) return 'success'
  if (status >= 400 && status < 500) return 'warning'
  if (status >= 500) return 'error'
  return ''
}
</script>

<style scoped>
.monitor-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 240px;
  background: #1a1a2e;
  border-radius: 8px;
  overflow: hidden;
}

.monitor-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background: #16213e;
  border-bottom: 1px solid #0f3460;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.pulse-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #666;
}

.pulse-dot.active {
  background: #00ff88;
  animation: pulse 1.5s infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

.monitor-header h3 {
  margin: 0;
  font-size: 14px;
  color: #fff;
}

.event-count {
  font-size: 11px;
  color: #888;
  padding: 2px 8px;
  background: #0f3460;
  border-radius: 10px;
}

.header-actions {
  display: flex;
  gap: 8px;
}

.btn-clear, .btn-toggle {
  padding: 4px 12px;
  font-size: 11px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-clear {
  background: #333;
  color: #aaa;
}

.btn-clear:hover {
  background: #444;
  color: #fff;
}

.btn-toggle {
  background: #00ff88;
  color: #000;
}

.btn-toggle.active {
  background: #ff6b6b;
  color: #fff;
}

.event-list {
  flex: 1;
  overflow-y: auto;
  padding: 12px;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: #666;
}

.empty-icon {
  font-size: 48px;
  margin-bottom: 12px;
  opacity: 0.5;
}

.empty-state p {
  margin: 4px 0;
  font-size: 13px;
}

.empty-state .hint {
  font-size: 11px;
  color: #555;
  margin-top: 8px;
}

.event-item {
  background: #16213e;
  border-radius: 6px;
  margin-bottom: 8px;
  overflow: hidden;
}

.event-item.llm_request {
  border-left: 3px solid #00ff88;
}

.event-item.llm_response {
  border-left: 3px solid #4ecdc4;
}

.event-item.llm_error {
  border-left: 3px solid #ff6b6b;
}

.event-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: rgba(0,0,0,0.2);
}

.event-type {
  font-size: 14px;
}

.event-time {
  font-size: 11px;
  color: #888;
  font-family: monospace;
}

.event-badge {
  font-size: 9px;
  padding: 2px 6px;
  border-radius: 3px;
  font-weight: bold;
}

.event-badge.llm_request {
  background: #00ff8833;
  color: #00ff88;
}

.event-badge.llm_response {
  background: #4ecdc433;
  color: #4ecdc4;
}

.event-badge.llm_error {
  background: #ff6b6b33;
  color: #ff6b6b;
}

.event-body {
  padding: 10px 12px;
}

.info-row {
  display: flex;
  gap: 8px;
  margin-bottom: 4px;
  font-size: 12px;
}

.info-row:last-child {
  margin-bottom: 0;
}

.info-row .label {
  color: #888;
  min-width: 70px;
}

.info-row .value {
  color: #fff;
  word-break: break-all;
}

.info-row .value.model {
  color: #4ecdc4;
}

.info-row .value.url {
  color: #888;
  font-size: 10px;
}

.info-row .value.success {
  color: #00ff88;
}

.info-row .value.warning {
  color: #ffd93d;
}

.info-row .value.error {
  color: #ff6b6b;
}

.error-message {
  color: #ff6b6b;
  font-size: 11px;
  margin-top: 4px;
  word-break: break-all;
}
</style>
