<script setup lang="ts">
/**
 * TraceHistory.vue - 历史任务表格
 */
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import TracePlayer from './TracePlayer.vue'

interface TraceRecord {
  id: string
  agent_id: string
  task_name: string
  status: string
  steps: number
  cost: number
  duration_sec: number
  created_at: string
}

const emit = defineEmits<{
  'select': [trace: TraceRecord]
}>()

const traces = ref<TraceRecord[]>([])
const loading = ref(false)

async function loadTraces() {
  loading.value = true
  try {
    const result: TraceRecord[] = await invoke('get_all_traces')
    // Plugin-sql returns rows directly
    traces.value = Array.isArray(result) ? result : []
  } catch (e) {
    // May fail if DB not initialized; show empty
    console.warn('get_all_traces failed:', e)
    traces.value = []
  }
  loading.value = false
}

function formatDuration(sec: number): string {
  if (sec < 60) return `${sec}s`
  const m = Math.floor(sec / 60)
  const s = sec % 60
  return `${m}m ${s}s`
}

function formatDate(iso: string): string {
  try {
    const d = new Date(iso)
    return d.toLocaleDateString('zh-CN', {
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    })
  } catch {
    return iso
  }
}

function statusClass(s: string): string {
  if (s === 'success') return 'success'
  if (s === 'failed') return 'failed'
  return ''
}

function statusLabel(s: string): string {
  if (s === 'success') return '成功'
  if (s === 'failed') return '失败'
  if (s === 'running') return '进行中'
  return s
}

// Demo data when DB is empty
const demoTraces: TraceRecord[] = [
  {
    id: 'trace-001',
    agent_id: 'agent-01',
    task_name: '整理发票',
    status: 'success',
    steps: 12,
    cost: 0.34,
    duration_sec: 720,
    created_at: new Date(Date.now() - 3600000).toISOString(),
  },
  {
    id: 'trace-002',
    agent_id: 'agent-03',
    task_name: '客服自动回复',
    status: 'success',
    steps: 28,
    cost: 1.89,
    duration_sec: 7200,
    created_at: new Date(Date.now() - 86400000).toISOString(),
  },
  {
    id: 'trace-003',
    agent_id: 'agent-02',
    task_name: '监控竞品网页',
    status: 'failed',
    steps: 4,
    cost: 0.12,
    duration_sec: 120,
    created_at: new Date(Date.now() - 172800000).toISOString(),
  },
  {
    id: 'trace-004',
    agent_id: 'agent-01',
    task_name: '数据报表导出',
    status: 'success',
    steps: 7,
    cost: 0.08,
    duration_sec: 180,
    created_at: new Date(Date.now() - 259200000).toISOString(),
  },
]

onMounted(async () => {
  await loadTraces()
  // Fall back to demo data if DB empty
  if (traces.value.length === 0) {
    traces.value = demoTraces
  }
})
</script>

<template>
  <div class="trace-history">
    <h3>📚 历史任务记录</h3>

    <div v-if="loading" class="trace-loading">
      加载中…
    </div>

    <table v-else class="trace-table">
      <thead>
        <tr>
          <th>任务名称</th>
          <th>Agent</th>
          <th>日期</th>
          <th>耗时</th>
          <th>步骤</th>
          <th>消耗</th>
          <th>状态</th>
          <th>操作</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="t in traces" :key="t.id">
          <td class="tt-name">{{ t.task_name }}</td>
          <td>{{ t.agent_id.toUpperCase() }}</td>
          <td>{{ formatDate(t.created_at) }}</td>
          <td>{{ formatDuration(t.duration_sec) }}</td>
          <td>{{ t.steps }}</td>
          <td class="cost-val">${{ t.cost.toFixed(2) }}</td>
          <td>
            <span class="tt-badge" :class="statusClass(t.status)">
              {{ statusLabel(t.status) }}
            </span>
          </td>
          <td>
            <span class="tt-play" @click="emit('select', t)">
              ▶ 回放
            </span>
          </td>
        </tr>

        <tr v-if="traces.length === 0">
          <td colspan="8" class="trace-empty">
            暂无历史记录
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<style scoped>
.trace-history {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 18px;
}

.trace-history h3 {
  font-size: 13px;
  font-weight: 600;
  margin-bottom: 14px;
}

.trace-table {
  width: 100%;
  border-collapse: collapse;
}

.trace-table th,
.trace-table td {
  text-align: left;
  padding: 10px 12px;
  border-bottom: 1px solid var(--border);
  font-size: 12px;
}

.trace-table th {
  color: var(--text-dim);
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  font-weight: 600;
}

.trace-table td {
  color: var(--text-secondary);
}

.trace-table tr:hover td {
  color: var(--text-primary);
  background: rgba(255, 255, 255, 0.01);
}

.trace-table .tt-name {
  font-weight: 600;
  color: var(--text-primary);
}

.tt-badge {
  font-size: 10px;
  padding: 2px 8px;
  border-radius: 10px;
  font-weight: 500;
}

.tt-badge.success {
  background: var(--green-glow);
  color: var(--green);
}

.tt-badge.failed {
  background: var(--red-glow);
  color: var(--red);
}

.tt-play {
  color: var(--cyan);
  cursor: pointer;
  font-size: 11px;
}

.tt-play:hover {
  text-decoration: underline;
}

.cost-val {
  font-family: var(--font-mono);
  color: var(--amber);
  font-weight: 600;
}

.trace-empty {
  text-align: center;
  color: var(--text-dim);
  padding: 32px !important;
}

.trace-loading {
  text-align: center;
  color: var(--text-dim);
  padding: 32px;
}
</style>
