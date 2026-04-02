<script setup lang="ts">
/**
 * ThoughtLog.vue - 思维流日志面板
 * 监听 proxyStore 的 thinkingLog 和 actionLog
 */
import { ref, computed, watch, nextTick } from 'vue'
import { useProxyStore } from '@/stores/proxy'

const proxyStore = useProxyStore()

type FilterType = 'all' | 'think' | 'action'

const activeFilter = ref<FilterType>('all')

const filters: { key: FilterType; label: string; color: string }[] = [
  { key: 'all',    label: 'ALL',   color: 'var(--text-dim)' },
  { key: 'think',  label: 'Think', color: '#a78bfa' },
  { key: 'action', label: 'Act',   color: 'var(--cyan)' },
]

// 合并 thinkingLog 和 actionLog 并按 step 排序
const mergedEvents = computed(() => {
  const events = [
    ...proxyStore.thinkingLog.map(e => ({ type: 'think', text: e.text, step: e.step, timestamp: e.timestamp || '--:--:--' })),
    ...proxyStore.actionLog.map(e => ({ type: 'action', text: `Call tool: ${e.tool}\n${JSON.stringify(e.params, null, 2)}`, step: e.step, timestamp: e.timestamp || '--:--:--' }))
  ]
  return events.sort((a, b) => a.step - b.step)
})

const filteredEvents = computed(() => {
  if (activeFilter.value === 'all') return mergedEvents.value
  return mergedEvents.value.filter(
    (e) => e.type === activeFilter.value
  )
})

// Auto-scroll to bottom
const logContainer = ref<HTMLElement | null>(null)

watch(
  () => mergedEvents.value.length,
  async () => {
    await nextTick()
    if (logContainer.value) {
      logContainer.value.scrollTop = logContainer.value.scrollHeight
    }
  }
)

function setFilter(f: FilterType) {
  activeFilter.value = f
}

function badgeClass(type: string | undefined) {
  return {
    'log-badge': true,
    think:   type === 'think',
    action:  type === 'action',
  }
}

function entryClass(type: string | undefined) {
  return {
    'log-entry': true,
    [`type-${type}`]: true,
  }
}

function renderContent(event: { text?: string }): string {
  if (!event.text) return ''
  return event.text.length > 200
    ? event.text.slice(0, 200) + '…'
    : event.text
}
</script>

<template>
  <div class="thought-stream">
    <!-- Header with filter chips -->
    <div class="thought-stream-header">
      <h3>🧠 思维流</h3>
      <div class="filter-chips">
        <button
          v-for="f in filters"
          :key="f.key"
          class="filter-chip"
          :class="{ active: activeFilter === f.key }"
          :style="activeFilter === f.key ? { borderColor: f.color, color: f.color, background: `color-mix(in srgb, ${f.color} 10%, transparent)` } : {}"
          @click="setFilter(f.key)"
        >
          {{ f.label }}
        </button>
      </div>
    </div>

    <!-- Log entries -->
    <div class="thought-log" ref="logContainer">
      <div
        v-if="filteredEvents.length === 0"
        class="log-empty"
      >
        <span>等待事件流…</span>
      </div>
      <div
        v-for="(event, idx) in filteredEvents"
        :key="idx"
        :class="entryClass(event.type)"
      >
        <span class="log-time">{{ event.timestamp || '--:--:--' }}</span>
        <span :class="badgeClass(event.type)">{{ event.type || 'unknown' }}</span>
        <span
          class="log-content"
          v-html="renderContent(event)"
        ></span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.thought-stream {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--bg-base);
}

.thought-stream-header {
  padding: 12px 16px;
  border-bottom: 1px solid var(--border);
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-shrink: 0;
}

.thought-stream-header h3 {
  font-size: 13px;
  font-weight: 600;
  display: flex;
  align-items: center;
  gap: 8px;
}

.filter-chips {
  display: flex;
  gap: 4px;
}

.filter-chip {
  font-size: 10px;
  padding: 3px 9px;
  border-radius: 10px;
  border: 1px solid var(--border);
  cursor: pointer;
  color: var(--text-dim);
  transition: all 0.2s;
  background: transparent;
  font-family: var(--font-ui);
  font-weight: 500;
}

.filter-chip:hover {
  border-color: var(--border-light);
  color: var(--text-secondary);
}

.thought-log {
  flex: 1;
  overflow-y: auto;
  padding: 6px 0;
  font-family: var(--font-mono);
  font-size: 12px;
}

.log-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--text-dim);
  font-size: 12px;
}

.log-entry {
  padding: 6px 16px;
  display: flex;
  gap: 10px;
  align-items: flex-start;
  transition: background 0.15s;
  border-left: 2px solid transparent;
}

.log-entry:hover {
  background: rgba(255, 255, 255, 0.02);
}

.log-entry.type-think  { border-left-color: #8b5cf6; }
.log-entry.type-action { border-left-color: var(--cyan); }
.log-entry.type-observe { border-left-color: var(--amber); }
.log-entry.type-error   { border-left-color: var(--red); background: var(--red-glow); }
.log-entry.type-unknown  { border-left-color: var(--text-dim); }

.log-time {
  color: var(--text-dim);
  font-size: 10px;
  min-width: 55px;
  padding-top: 2px;
  flex-shrink: 0;
}

.log-badge {
  font-size: 9px;
  padding: 1px 6px;
  border-radius: 3px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  min-width: 52px;
  text-align: center;
  flex-shrink: 0;
}

.log-badge.think   { background: rgba(139, 92, 246, 0.2); color: #a78bfa; }
.log-badge.action  { background: var(--cyan-glow); color: var(--cyan); }
.log-badge.observe { background: var(--amber-glow); color: var(--amber); }
.log-badge.error   { background: var(--red-glow); color: var(--red); }
.log-badge.unknown { background: var(--bg-elevated); color: var(--text-dim); }

.log-content {
  flex: 1;
  color: var(--text-secondary);
  line-height: 1.5;
  word-break: break-word;
}

/* Highlight tool calls rendered from v-html */
.log-content :deep(.highlight) {
  color: var(--cyan);
  background: var(--cyan-glow);
  padding: 0 4px;
  border-radius: 3px;
}

.log-content :deep(.warn-hl) {
  color: var(--amber);
}
</style>
