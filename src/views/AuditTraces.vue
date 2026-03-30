<template>
  <div class="audit-traces">
    <div class="audit-header">
      <h1>📋 Audit & Traces</h1>
      <div class="audit-actions">
        <button @click="exportLogs('json')" class="btn-export">Export JSON</button>
        <button @click="exportLogs('csv')" class="btn-export">Export CSV</button>
      </div>
    </div>
    
    <!-- Cost Summary -->
    <div class="cost-summary">
      <div class="cost-card">
        <div class="cost-label">Today's Cost</div>
        <div class="cost-value">${{ todayCost.toFixed(4) }}</div>
      </div>
      <div class="cost-card">
        <div class="cost-label">This Week</div>
        <div class="cost-value">${{ weekCost.toFixed(4) }}</div>
      </div>
      <div class="cost-card">
        <div class="cost-label">Budget Remaining</div>
        <div class="cost-value" :class="{ warning: budgetRemaining < 1 }">
          ${{ budgetRemaining.toFixed(2) }}
        </div>
        <div class="budget-bar">
          <div 
            class="budget-fill" 
            :style="{ width: `${budgetPercent}%` }"
            :class="{ critical: budgetPercent < 20 }"
          ></div>
        </div>
      </div>
    </div>
    
    <!-- Filters -->
    <div class="audit-filters">
      <select v-model="filterAgent" class="filter-select">
        <option value="">All Agents</option>
        <option v-for="a in agents" :key="a" :value="a">{{ a }}</option>
      </select>
      <select v-model="filterAction" class="filter-select">
        <option value="">All Actions</option>
        <option v-for="a in actionTypes" :key="a" :value="a">{{ a }}</option>
      </select>
      <select v-model="filterTime" class="filter-select">
        <option value="today">Today</option>
        <option value="week">This Week</option>
        <option value="month">This Month</option>
        <option value="all">All Time</option>
      </select>
      <input 
        v-model="searchQuery" 
        type="text" 
        placeholder="Search logs..."
        class="filter-search"
      />
    </div>
    
    <!-- Audit Table -->
    <div class="audit-table-container">
      <table class="audit-table">
        <thead>
          <tr>
            <th>Time</th>
            <th>Agent</th>
            <th>Action</th>
            <th>Details</th>
            <th>Cost</th>
            <th>HITL</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="entry in filteredEntries" :key="entry.id" @click="selectEntry(entry)">
            <td class="time-cell">{{ formatTime(entry.timestamp) }}</td>
            <td class="agent-cell">{{ entry.agent_id }}</td>
            <td class="action-cell">
              <span :class="['action-badge', entry.action_type.toLowerCase()]">
                {{ formatAction(entry.action_type) }}
              </span>
            </td>
            <td class="details-cell">{{ truncate(entry.action_detail, 50) }}</td>
            <td class="cost-cell">${{ entry.cost_usd.toFixed(4) }}</td>
            <td class="hitl-cell">
              <span v-if="entry.hitl_approved === true" class="hitl-approved">✅</span>
              <span v-else-if="entry.hitl_approved === false" class="hitl-rejected">❌</span>
              <span v-else class="hitl-na">-</span>
            </td>
          </tr>
        </tbody>
      </table>
      
      <div v-if="filteredEntries.length === 0" class="empty-state">
        No audit logs found
      </div>
    </div>
    
    <!-- Detail Panel -->
    <div v-if="selectedEntry" class="detail-panel">
      <div class="detail-header">
        <h3>Audit Entry Details</h3>
        <button @click="selectedEntry = null" class="btn-close">×</button>
      </div>
      <div class="detail-content">
        <div class="detail-row">
          <span class="detail-label">ID:</span>
          <span class="detail-value">{{ selectedEntry.id }}</span>
        </div>
        <div class="detail-row">
          <span class="detail-label">Timestamp:</span>
          <span class="detail-value">{{ selectedEntry.timestamp }}</span>
        </div>
        <div class="detail-row">
          <span class="detail-label">Agent:</span>
          <span class="detail-value">{{ selectedEntry.agent_id }}</span>
        </div>
        <div class="detail-row">
          <span class="detail-label">Session:</span>
          <span class="detail-value">{{ selectedEntry.session_id }}</span>
        </div>
        <div class="detail-row">
          <span class="detail-label">Action:</span>
          <span class="detail-value">{{ selectedEntry.action_type }}</span>
        </div>
        <div class="detail-row full-width">
          <span class="detail-label">Details:</span>
          <pre class="detail-json">{{ JSON.stringify(selectedEntry.action_detail, null, 2) }}</pre>
        </div>
        <div class="detail-row">
          <span class="detail-label">Cost:</span>
          <span class="detail-value">${{ selectedEntry.cost_usd.toFixed(6) }}</span>
        </div>
        <div class="detail-row">
          <span class="detail-label">Hash:</span>
          <span class="detail-value hash">{{ selectedEntry.hash?.substring(0, 16) }}...</span>
        </div>
      </div>
      <div class="detail-actions">
        <button @click="verifyIntegrity(selectedEntry)" class="btn-verify">
          Verify Integrity
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

interface AuditEntry {
  id: string
  timestamp: string
  agent_id: string
  session_id: string
  action_type: string
  action_detail: any
  cost_usd: number
  hitl_approved: boolean | null
  hash: string
}

const entries = ref<AuditEntry[]>([])
const selectedEntry = ref<AuditEntry | null>(null)
const todayCost = ref(0)
const weekCost = ref(0)
const budgetRemaining = ref(5.00)
const budget = ref(5.00)

const filterAgent = ref('')
const filterAction = ref('')
const filterTime = ref('today')
const searchQuery = ref('')

const actionTypes = ['Screenshot', 'MouseClick', 'KeyPress', 'BashExec', 'FileWrite', 'AgentStart', 'AgentStop', 'HitlApproval']

const agents = computed(() => {
  const set = new Set(entries.value.map(e => e.agent_id))
  return Array.from(set)
})

const budgetPercent = computed(() => {
  return (budgetRemaining.value / budget.value) * 100
})

const filteredEntries = computed(() => {
  let filtered = entries.value
  
  if (filterAgent.value) {
    filtered = filtered.filter(e => e.agent_id === filterAgent.value)
  }
  
  if (filterAction.value) {
    filtered = filtered.filter(e => e.action_type === filterAction.value)
  }
  
  if (searchQuery.value) {
    const q = searchQuery.value.toLowerCase()
    filtered = filtered.filter(e => 
      JSON.stringify(e.action_detail).toLowerCase().includes(q)
    )
  }
  
  return filtered
})

onMounted(async () => {
  await loadEntries()
  await loadCostSummary()
})

async function loadEntries() {
  try {
    entries.value = await invoke('get_audit_logs', {
      filter: {
        agent_id: filterAgent.value || null,
        action_type: filterAction.value || null,
        time_range: filterTime.value,
      }
    })
  } catch (error) {
    console.error('Failed to load audit logs:', error)
    // Mock data for development
    entries.value = [
      {
        id: '1',
        timestamp: new Date().toISOString(),
        agent_id: 'invoice-bot',
        session_id: 'sess-001',
        action_type: 'Screenshot',
        action_detail: { width: 1280, height: 800 },
        cost_usd: 0.0048,
        hitl_approved: null,
        hash: 'abc123def456',
      },
      {
        id: '2',
        timestamp: new Date(Date.now() - 60000).toISOString(),
        agent_id: 'invoice-bot',
        session_id: 'sess-001',
        action_type: 'MouseClick',
        action_detail: { x: 342, y: 120, button: 'left' },
        cost_usd: 0.001,
        hitl_approved: true,
        hash: 'def789ghi012',
      },
      {
        id: '3',
        timestamp: new Date(Date.now() - 120000).toISOString(),
        agent_id: 'monitor-agent',
        session_id: 'sess-002',
        action_type: 'BashExec',
        action_detail: { command: 'ls -la /tmp' },
        cost_usd: 0.002,
        hitl_approved: false,
        hash: 'ghi345jkl678',
      },
    ]
  }
}

async function loadCostSummary() {
  try {
    const summary = await invoke('get_cost_summary')
    todayCost.value = summary.today || 0
    weekCost.value = summary.week || 0
    budgetRemaining.value = summary.budget_remaining || 5.00
  } catch (error) {
    console.error('Failed to load cost summary:', error)
    todayCost.value = 0.047
    weekCost.value = 0.234
  }
}

async function exportLogs(format: string) {
  try {
    const data = await invoke('export_audit_logs', { format })
    // Download the file
    const blob = new Blob([data], { type: format === 'json' ? 'application/json' : 'text/csv' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `audit-logs.${format}`
    a.click()
    URL.revokeObjectURL(url)
  } catch (error) {
    console.error('Failed to export logs:', error)
  }
}

async function verifyIntegrity(entry: AuditEntry) {
  try {
    const valid = await invoke('verify_audit_integrity', { entryId: entry.id })
    alert(valid ? '✅ Entry integrity verified!' : '❌ Entry has been tampered with!')
  } catch (error) {
    console.error('Failed to verify integrity:', error)
    alert('Verification failed')
  }
}

function selectEntry(entry: AuditEntry) {
  selectedEntry.value = entry
}

function formatTime(timestamp: string): string {
  return new Date(timestamp).toLocaleTimeString()
}

function formatAction(action: string): string {
  const icons: Record<string, string> = {
    'Screenshot': '📷',
    'MouseClick': '🖱',
    'MouseMove': '🖱',
    'KeyPress': '⌨',
    'BashExec': '💻',
    'FileWrite': '📝',
    'FileRead': '📄',
    'AgentStart': '▶',
    'AgentStop': '⏹',
    'HitlApproval': '🔐',
  }
  return `${icons[action] || '🔧'} ${action}`
}

function truncate(obj: any, len: number): string {
  const str = typeof obj === 'string' ? obj : JSON.stringify(obj)
  return str.length > len ? str.substring(0, len) + '...' : str
}
</script>

<style scoped>
.audit-traces {
  padding: 1.5rem;
  display: grid;
  grid-template-columns: 1fr 350px;
  grid-template-rows: auto auto auto 1fr;
  gap: 1.5rem;
  height: calc(100vh - 4rem);
}

.audit-header {
  grid-column: 1 / -1;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.audit-header h1 {
  font-size: 1.5rem;
  font-weight: 600;
}

.audit-actions {
  display: flex;
  gap: 0.5rem;
}

.btn-export {
  padding: 0.5rem 1rem;
  background: rgba(59, 130, 246, 0.2);
  border: 1px solid rgba(59, 130, 246, 0.3);
  border-radius: 6px;
  color: #3b82f6;
  cursor: pointer;
}

.cost-summary {
  grid-column: 1 / -1;
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 1rem;
}

.cost-card {
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 12px;
  padding: 1.25rem;
}

.cost-label {
  font-size: 0.75rem;
  color: #888;
  margin-bottom: 0.5rem;
}

.cost-value {
  font-size: 1.5rem;
  font-weight: 600;
}

.cost-value.warning {
  color: #ef4444;
}

.budget-bar {
  margin-top: 0.75rem;
  height: 6px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 3px;
  overflow: hidden;
}

.budget-fill {
  height: 100%;
  background: #22c55e;
  transition: width 0.3s ease;
}

.budget-fill.critical {
  background: #ef4444;
  animation: pulse 2s infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

.audit-filters {
  grid-column: 1 / -1;
  display: flex;
  gap: 0.75rem;
}

.filter-select, .filter-search {
  padding: 0.5rem 1rem;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 6px;
  color: white;
}

.filter-select {
  min-width: 150px;
}

.filter-search {
  flex: 1;
  max-width: 300px;
}

.audit-table-container {
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 12px;
  overflow: hidden;
}

.audit-table {
  width: 100%;
  border-collapse: collapse;
}

.audit-table th {
  text-align: left;
  padding: 1rem;
  background: rgba(255, 255, 255, 0.05);
  font-weight: 500;
  font-size: 0.75rem;
  color: #888;
  text-transform: uppercase;
}

.audit-table td {
  padding: 0.875rem 1rem;
  border-top: 1px solid rgba(255, 255, 255, 0.05);
  font-size: 0.875rem;
}

.audit-table tbody tr {
  cursor: pointer;
  transition: background 0.2s;
}

.audit-table tbody tr:hover {
  background: rgba(59, 130, 246, 0.1);
}

.time-cell {
  color: #888;
  font-size: 0.75rem;
}

.agent-cell {
  font-weight: 500;
}

.action-badge {
  padding: 0.25rem 0.5rem;
  border-radius: 4px;
  font-size: 0.75rem;
}

.action-badge.screenshot { background: rgba(168, 85, 247, 0.2); color: #a855f7; }
.action-badge.mouseclick { background: rgba(59, 130, 246, 0.2); color: #3b82f6; }
.action-badge.keypress { background: rgba(34, 197, 94, 0.2); color: #22c55e; }
.action-badge.bashexec { background: rgba(234, 179, 8, 0.2); color: #eab308; }
.action-badge.filewrite { background: rgba(239, 68, 68, 0.2); color: #ef4444; }

.cost-cell {
  font-family: monospace;
  color: #22c55e;
}

.hitl-approved { color: #22c55e; }
.hitl-rejected { color: #ef4444; }
.hitl-na { color: #666; }

.empty-state {
  padding: 3rem;
  text-align: center;
  color: #666;
}

.detail-panel {
  grid-row: span 3;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 12px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.detail-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1rem 1.25rem;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
}

.detail-header h3 {
  font-size: 1rem;
  font-weight: 500;
}

.btn-close {
  background: none;
  border: none;
  color: #888;
  font-size: 1.5rem;
  cursor: pointer;
}

.detail-content {
  flex: 1;
  overflow-y: auto;
  padding: 1rem;
}

.detail-row {
  display: flex;
  gap: 0.5rem;
  margin-bottom: 0.75rem;
  font-size: 0.875rem;
}

.detail-row.full-width {
  flex-direction: column;
}

.detail-label {
  color: #888;
  min-width: 80px;
}

.detail-value {
  color: #e0e0e0;
}

.detail-value.hash {
  font-family: monospace;
  font-size: 0.75rem;
  color: #888;
}

.detail-json {
  background: rgba(0, 0, 0, 0.3);
  padding: 0.75rem;
  border-radius: 6px;
  font-family: monospace;
  font-size: 0.75rem;
  overflow-x: auto;
  white-space: pre-wrap;
  word-break: break-all;
}

.detail-actions {
  padding: 1rem;
  border-top: 1px solid rgba(255, 255, 255, 0.1);
}

.btn-verify {
  width: 100%;
  padding: 0.75rem;
  background: rgba(34, 197, 94, 0.2);
  border: 1px solid rgba(34, 197, 94, 0.3);
  border-radius: 6px;
  color: #22c55e;
  cursor: pointer;
}
</style>