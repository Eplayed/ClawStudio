<script setup lang="ts">
/**
 * Agents.vue - 特工列队页面
 * 看板视图 + 通道聚合器 + 创建 Modal
 */
import { ref, computed, onMounted } from 'vue'
import { useAgentStore, type Agent } from '@/stores/agents'
import { invoke } from '@tauri-apps/api/core'
import KanbanBoard from '@/components/KanbanBoard.vue'
import ChannelAggregator from '@/components/ChannelAggregator.vue'
import CreateAgentModal from '@/components/CreateAgentModal.vue'

const agentStore = useAgentStore()

// Sandbox list (for CreateAgentModal)
const sandboxes = ref<Array<{ id: string; name: string; vnc_port: number; created_at?: string }>>([])

async function loadSandboxes() {
  try {
    const raw: Array<{
      id: string
      name: string
      vnc_port: number
      created_at?: string
    }> = await invoke('list_sandboxes')
    sandboxes.value = raw.map(s => ({
      id: s.id,
      name: s.name,
      vnc_port: s.vnc_port,
      created_at: s.created_at,
    }))
  } catch (e) {
    // Docker may not be available
    sandboxes.value = []
  }
}

// Modal state
const showCreateModal = ref(false)

// Demo agents (replace with DB load later)
const demoAgents: Agent[] = [
  {
    id: 'agent-01',
    name: '整理发票',
    avatar: '📄',
    systemPrompt: '自动识别并整理本月 PDF 发票，提取金额汇总到表格',
    computerUse: true,
    networkAccess: true,
    sandboxId: 'sb-01',
    fileWhitelist: '',
    budgetLimit: 1.0,
    tokenLimit: 100000,
    status: 'running',
    currentCost: 0.34,
    tokensUsed: 68200,
    elapsedSec: 720,
    progress: 67,
  },
  {
    id: 'agent-02',
    name: '监控竞品网页',
    avatar: '🔍',
    systemPrompt: '每2小时巡查竞品官网价格变动，截图存档',
    computerUse: false,
    networkAccess: true,
    sandboxId: null,
    fileWhitelist: '',
    budgetLimit: 0.5,
    tokenLimit: 50000,
    status: 'paused',
    currentCost: 0.12,
    tokensUsed: 24100,
    elapsedSec: 0,
    progress: 100,
  },
  {
    id: 'agent-03',
    name: '客服自动回复',
    avatar: '💬',
    systemPrompt: '监听 Telegram 群组消息，AI 自动响应客户咨询',
    computerUse: false,
    networkAccess: true,
    sandboxId: null,
    fileWhitelist: '',
    budgetLimit: 5.0,
    tokenLimit: 200000,
    status: 'running',
    currentCost: 1.89,
    tokensUsed: 117500,
    elapsedSec: 7200,
    progress: 40,
  },
  {
    id: 'agent-04',
    name: '批量发送周报',
    avatar: '📧',
    systemPrompt: '从模板生成周报并发送至团队邮箱',
    computerUse: false,
    networkAccess: true,
    sandboxId: null,
    fileWhitelist: '',
    budgetLimit: 0.2,
    tokenLimit: 20000,
    status: 'queued',
    currentCost: 0,
    tokensUsed: 0,
    elapsedSec: 0,
    progress: 0,
  },
]

// Merge demo + store agents
const allAgents = computed(() => {
  const storeIds = new Set(agentStore.agents.map(a => a.id))
  const extras = demoAgents.filter(a => !storeIds.has(a.id))
  return [...agentStore.agents, ...extras]
})

const activeCount = computed(() => allAgents.value.filter(a => a.status === 'running').length)

function handleCardClick(agent: Agent) {
  console.log('Agent clicked:', agent.id)
  // Navigate to overwatch
}

function handleAgentCreated(agent: Agent) {
  agentStore.addAgent(agent)
}

onMounted(async () => {
  await loadSandboxes()
  // Register demo agents
  for (const a of demoAgents) {
    if (!agentStore.agents.find(e => e.id === a.id)) {
      agentStore.addAgent(a)
    }
  }
})
</script>

<template>
  <div class="page">
    <!-- Top bar -->
    <header class="topbar">
      <div class="page-title">
        <span>🤖</span> 特工列队 Agents
      </div>
      <div class="topbar-actions">
        <div class="status-chip">
          <div class="dot"></div>
          {{ activeCount }} 活跃
        </div>
      </div>
    </header>

    <!-- Content -->
    <div class="agents-content">
      <!-- Section: Task Board -->
      <div class="section-header">
        <h2>📋 任务看板 Task Board</h2>
      </div>

      <KanbanBoard
        :agents="allAgents"
        @card-click="handleCardClick"
      />

      <!-- Section: Channel Aggregator -->
      <div class="section-header">
        <h2>📡 通道聚合器 Channel Aggregator</h2>
      </div>

      <ChannelAggregator />
    </div>

    <!-- FAB: Create new agent -->
    <button class="fab" @click="showCreateModal = true">
      +
    </button>

    <!-- Create Agent Modal -->
    <Teleport to="body">
      <CreateAgentModal
        v-if="showCreateModal"
        :sandboxes="sandboxes"
        @close="showCreateModal = false"
        @created="handleAgentCreated"
      />
    </Teleport>
  </div>
</template>

<style scoped>
.page {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  position: relative;
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
  display: flex;
  align-items: center;
  gap: 8px;
}

.topbar-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.status-chip {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 12px;
  border-radius: 20px;
  font-size: 11px;
  font-weight: 500;
  background: var(--green-glow);
  color: var(--green);
  border: 1px solid rgba(34, 197, 94, 0.2);
}

.dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--green);
  animation: pulse-dot 2s infinite;
}

@keyframes pulse-dot {
  0%, 100% { opacity: 1; box-shadow: 0 0 0 0 rgba(34, 197, 94, 0.5); }
  50% { opacity: 0.7; box-shadow: 0 0 0 4px rgba(34, 197, 94, 0); }
}

.agents-content {
  flex: 1;
  overflow-y: auto;
  padding: 20px 24px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 14px;
}

.section-header h2 {
  font-size: 14px;
  font-weight: 600;
}

/* FAB */
.fab {
  position: fixed;
  bottom: 24px;
  right: 24px;
  width: 48px;
  height: 48px;
  border-radius: 50%;
  background: linear-gradient(135deg, var(--cyan), #0891b2);
  border: none;
  color: #fff;
  font-size: 22px;
  cursor: pointer;
  box-shadow: 0 4px 20px rgba(6, 214, 214, 0.3);
  transition: all 0.3s;
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 50;
}

.fab:hover {
  transform: scale(1.1);
  box-shadow: 0 4px 30px rgba(6, 214, 214, 0.5);
}
</style>
