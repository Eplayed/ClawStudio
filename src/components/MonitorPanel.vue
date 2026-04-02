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
        <p>Waiting for LLM requests...</p>
        <p class="hint">Start QClaw with the hijack script to see events</p>
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
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, nextTick } from 'vue'
import { useProxyStore } from '@/stores/proxy'
import type { MonitorEvent, LLMRequestEvent, LLMResponseEvent, LLMErrorEvent } from '@/stores/proxy'

const store = useProxyStore()
const eventListRef = ref<HTMLElement | null>(null)
const isListening = ref(true)

onMounted(async () => {
  // 初始化 store（如果还没初始化）
  if (store.monitorEvents.length === 0) {
    await store.init()
  }
})

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
    default: return '📝'
  }
}

function getEventLabel(type: string): string {
  switch (type) {
    case 'llm_request': return 'REQUEST'
    case 'llm_response': return 'RESPONSE'
    case 'llm_error': return 'ERROR'
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
