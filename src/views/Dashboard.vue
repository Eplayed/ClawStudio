<script setup lang="ts">
import { computed, ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAgentStore } from '@/stores/agents'
import { invoke } from '@tauri-apps/api/core'
import GatewayStatusBar from '@/components/GatewayStatusBar.vue'
import MonitorPanel from '@/components/MonitorPanel.vue'

const router = useRouter()
const agentStore = useAgentStore()

const totalCost = computed(() => agentStore.totalCostToday.toFixed(2))

// Real agents from store, fallback to mock for display
const displayAgents = computed(() => {
  if (agentStore.agents.length > 0) {
    return agentStore.agents.map(a => ({
      id: a.id,
      name: a.name,
      icon: a.avatar,
      status: a.status,
      cost: a.currentCost,
      time: a.elapsedSec > 0 ? `${Math.floor(a.elapsedSec / 60)}m` : '新启动',
      progress: a.progress,
      task: a.systemPrompt?.slice(0, 50) || '执行任务'
    }))
  }
  // Mock data for initial display
  return [
    { id: 'agent-01', name: '整理发票', icon: '📄', status: 'running', cost: 0.34, time: '12m', progress: 67, task: '自动识别并整理本月 PDF 发票' },
    { id: 'agent-02', name: '监控竞品网页', icon: '🔍', status: 'sleeping', cost: 0.12, time: '每2h', progress: 100, task: '每2小时巡查竞品官网价格变动' },
    { id: 'agent-03', name: '客服自动回复', icon: '💬', status: 'running', cost: 1.89, time: '24条', progress: 40, task: '监听 Telegram 群组消息，自动响应' },
  ]
})

// Real activity from event log
const recentActivities = computed(() => {
  if (agentStore.eventLog.length > 0) {
    return agentStore.eventLog.slice(-6).reverse().map(e => ({
      time: e.timestamp?.slice(11, 16) || '刚刚',
      icon: e.tool === 'screenshot' ? '📸' : e.tool === 'bash' ? '💻' : e.type === 'hitl' ? '🚨' : '✅',
      text: `<strong>${e.agent_id || 'Agent'}</strong> ${e.tool || e.type || '操作'}`
    }))
  }
  // Mock fallback
  return [
    { time: '14:32', icon: '✅', text: '<strong>AGENT-01</strong> 完成任务：发票整理' },
    { time: '14:28', icon: '🚨', text: '<strong>AGENT-03</strong> 触发 HITL 审批' },
    { time: '14:15', icon: '💬', text: '<strong>AGENT-03</strong> 发送消息到 Telegram' },
    { time: '14:02', icon: '🔍', text: '<strong>AGENT-02</strong> 完成竞品价格巡查' },
    { time: '13:45', icon: '📸', text: '<strong>AGENT-01</strong> 截图分析完成' },
    { time: '13:30', icon: '🤖', text: '<strong>AGENT-01</strong> 开始执行任务' },
  ]
})

// Cost trend - load from backend
const costData = ref([
  { day: '周一', cost: 1.23 },
  { day: '周二', cost: 1.50 },
  { day: '周三', cost: 1.15 },
  { day: '周四', cost: 1.87 },
  { day: '周五', cost: 2.04 },
  { day: '周六', cost: 0.55 },
  { day: '今日', cost: 2.47, highlight: true },
])

async function loadCostTrend() {
  try {
    const summary = await invoke<any>('get_cost_summary', { days: 7 })
    if (summary?.daily) {
      costData.value = summary.daily.map((d: any, i: number) => ({
        day: d.day || `Day ${i + 1}`,
        cost: d.cost || 0,
        highlight: i === summary.daily.length - 1
      }))
    }
  } catch (e) {
    // Keep mock data
  }
}

const maxCost = computed(() => Math.max(...costData.value.map(d => d.cost)))

onMounted(() => {
  loadCostTrend()
})
</script>

<template>
  <div class="page">
    <header class="topbar">
      <div class="page-title">📊 仪表盘 Dashboard</div>
      <div class="status-chip">
        <div class="dot"></div>
        OpenClaw Core 运行中
      </div>
    </header>

    <div class="content">
      <!-- Proxy Status Bar -->
      <GatewayStatusBar />

      <!-- Monitor Panel -->
      <div class="monitor-wrap">
        <MonitorPanel />
      </div>

      <!-- Stats Row -->
      <div class="stats-row">
        <div class="stat-card cyan">
          <div class="stat-label">活跃特工 Active Agents</div>
          <div class="stat-value">{{ displayAgents.filter(a => a.status === 'running').length }}</div>
          <div class="stat-sub">{{ displayAgents.length }} 总计</div>
        </div>
        <div class="stat-card amber">
          <div class="stat-label">今日任务 Tasks Today</div>
          <div class="stat-value">12</div>
          <div class="stat-sub">9 已完成 / 3 进行中</div>
        </div>
        <div class="stat-card red">
          <div class="stat-label">今日消耗 API Cost</div>
          <div class="stat-value">${{ totalCost || '2.47' }}</div>
          <div class="stat-sub">截图 Token 占比 68%</div>
        </div>
        <div class="stat-card green">
          <div class="stat-label">系统状态 Health</div>
          <div class="stat-value">GOOD</div>
          <div class="stat-sub">Docker ✓ | CPU 23% | RAM 1.2G</div>
        </div>
      </div>

      <!-- Active Agents -->
      <div class="section-header">
        <h2>🤖 活跃特工</h2>
        <span class="see-all" @click="router.push('/agents')">查看全部 →</span>
      </div>
      <div class="agent-grid">
        <div
          v-for="agent in displayAgents"
          :key="agent.id"
          class="agent-card"
          @click="router.push('/overwatch/' + agent.id)"
        >
          <div class="card-header">
            <span class="agent-id">{{ agent.id.toUpperCase() }}</span>
            <span class="status-badge" :class="agent.status">
              <span class="sdot"></span>
              {{ agent.status === 'running' ? '运行中' : '休眠中' }}
            </span>
          </div>
          <div class="agent-name">{{ agent.icon }} {{ agent.name }}</div>
          <div class="agent-task">{{ agent.task }}</div>
          <div class="agent-stats">
            <span>💰 ${{ agent.cost.toFixed(2) }}</span>
            <span>⏱ {{ agent.time }}</span>
            <span>🎯 {{ agent.progress }}%</span>
          </div>
          <div class="progress-bar">
            <div class="fill" :style="{ width: agent.progress + '%' }"></div>
          </div>
        </div>
      </div>

      <!-- D1 + D2: Activity & Cost Trend -->
      <div class="bottom-grid">
        <!-- Recent Activity -->
        <div class="activity-panel">
          <div class="section-header">
            <h2>🕐 最近动态 Recent Activity</h2>
          </div>
          <div class="activity-list">
            <div
              v-for="(item, idx) in recentActivities"
              :key="idx"
              class="activity-item"
            >
              <span class="act-time">{{ item.time }}</span>
              <span class="act-icon">{{ item.icon }}</span>
              <span class="act-text" v-html="item.text"></span>
            </div>
          </div>
        </div>

        <!-- Cost Trend -->
        <div class="cost-trend-panel">
          <div class="section-header">
            <h2>📈 费用趋势 Cost Trend (7天)</h2>
          </div>
          <div class="cost-bars">
            <div
              v-for="(d, idx) in costData"
              :key="idx"
              class="cost-bar-col"
            >
              <div class="bar-wrapper">
                <div
                  class="bar"
                  :class="{ highlight: d.highlight }"
                  :style="{ height: (d.cost / maxCost * 100) + 'px' }"
                >
                  <span class="bar-tooltip">${{ d.cost.toFixed(2) }}</span>
                </div>
              </div>
              <span class="bar-label">{{ d.day }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.page { flex: 1; display: flex; flex-direction: column; overflow: hidden; }
.topbar {
  height: 52px; min-height: 52px;
  display: flex; align-items: center; justify-content: space-between;
  padding: 0 24px; border-bottom: 1px solid var(--border); background: var(--bg-base);
}
.page-title { font-size: 15px; font-weight: 600; }
.status-chip {
  display: flex; align-items: center; gap: 6px;
  padding: 4px 12px; border-radius: 20px; font-size: 11px; font-weight: 500;
  background: var(--green-glow); color: var(--green); border: 1px solid rgba(34,197,94,0.2);
}
.dot { width: 6px; height: 6px; border-radius: 50%; background: var(--green); animation: pulse-dot 2s infinite; }

.content { flex: 1; overflow-y: auto; padding: 20px 24px; }

.monitor-wrap { height: 260px; margin: 14px 0 20px; }

.stats-row { display: grid; grid-template-columns: repeat(4, 1fr); gap: 14px; margin-bottom: 20px; }
.stat-card {
  background: var(--bg-card); border: 1px solid var(--border); border-radius: var(--radius);
  padding: 16px 18px; position: relative; overflow: hidden;
}
.stat-card::after { content:''; position:absolute; top:0; left:0; right:0; height:2px; }
.stat-card.cyan::after { background: linear-gradient(90deg, var(--cyan), transparent); }
.stat-card.cyan .stat-value { color: var(--cyan); }
.stat-card.amber::after { background: linear-gradient(90deg, var(--amber), transparent); }
.stat-card.amber .stat-value { color: var(--amber); }
.stat-card.green::after { background: linear-gradient(90deg, var(--green), transparent); }
.stat-card.green .stat-value { color: var(--green); }
.stat-card.red::after { background: linear-gradient(90deg, var(--red), transparent); }
.stat-card.red .stat-value { color: var(--red); }
.stat-label { font-size: 11px; color: var(--text-dim); text-transform: uppercase; letter-spacing: 0.5px; margin-bottom: 8px; }
.stat-value { font-size: 26px; font-weight: 700; font-family: var(--font-mono); }
.stat-sub { font-size: 11px; color: var(--text-secondary); margin-top: 4px; }

.section-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 14px; }
.section-header h2 { font-size: 14px; font-weight: 600; }
.see-all { font-size: 11px; color: var(--cyan); cursor: pointer; }

.agent-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 14px; margin-bottom: 24px; }
.agent-card {
  background: var(--bg-card); border: 1px solid var(--border); border-radius: var(--radius);
  padding: 16px; cursor: pointer; transition: all 0.25s;
}
.agent-card:hover { border-color: var(--cyan-dim); box-shadow: 0 0 25px var(--cyan-glow); transform: translateY(-1px); }
.card-header { display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 12px; }
.agent-id { font-size: 10px; font-family: var(--font-mono); color: var(--text-dim); }
.status-badge {
  display: flex; align-items: center; gap: 5px; font-size: 11px; font-weight: 500;
  padding: 2px 8px; border-radius: 12px;
}
.status-badge.running { background: var(--green-glow); color: var(--green); }
.status-badge.sleeping { background: var(--amber-glow); color: var(--amber); }
.sdot { width: 6px; height: 6px; border-radius: 50%; background: currentColor; }
.status-badge.running .sdot { animation: pulse-dot 2s infinite; }
.agent-name { font-size: 14px; font-weight: 600; margin-bottom: 4px; }
.agent-task { font-size: 12px; color: var(--text-secondary); margin-bottom: 14px; }
.agent-stats { display: flex; gap: 16px; font-size: 11px; color: var(--text-dim); }
.progress-bar { height: 3px; background: var(--border); border-radius: 2px; margin-top: 12px; overflow: hidden; }
.progress-bar .fill { height: 100%; border-radius: 2px; background: linear-gradient(90deg, var(--cyan), var(--green)); transition: width 1s; }

/* D1 + D2: Bottom Grid */
.bottom-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 14px;
}

.activity-panel,
.cost-trend-panel {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 16px;
}

/* Activity List */
.activity-list {
  display: flex;
  flex-direction: column;
}

.activity-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 0;
  border-bottom: 1px solid var(--border);
}

.activity-item:last-child {
  border-bottom: none;
}

.act-time {
  font-family: var(--font-mono);
  font-size: 10px;
  color: var(--text-dim);
  min-width: 40px;
}

.act-icon {
  font-size: 14px;
}

.act-text {
  font-size: 12px;
  color: var(--text-secondary);
}

.act-text :deep(strong) {
  color: var(--text-primary);
  font-weight: 600;
}

/* Cost Bars */
.cost-bars {
  display: flex;
  justify-content: space-around;
  align-items: flex-end;
  height: 130px;
  padding-top: 20px;
}

.cost-bar-col {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
}

.bar-wrapper {
  height: 100px;
  display: flex;
  align-items: flex-end;
}

.bar {
  width: 28px;
  border-radius: 4px 4px 0 0;
  background: linear-gradient(180deg, var(--cyan), var(--cyan-dim));
  position: relative;
  cursor: pointer;
  transition: all 0.3s;
}

.bar:hover {
  filter: brightness(1.2);
}

.bar.highlight {
  background: linear-gradient(180deg, var(--amber), var(--amber-dim));
}

.bar-tooltip {
  position: absolute;
  bottom: 100%;
  left: 50%;
  transform: translateX(-50%);
  padding: 3px 8px;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: 4px;
  font-size: 10px;
  font-family: var(--font-mono);
  color: var(--text-primary);
  white-space: nowrap;
  opacity: 0;
  pointer-events: none;
  transition: opacity 0.2s;
}

.bar:hover .bar-tooltip {
  opacity: 1;
}

.bar-label {
  font-size: 10px;
  color: var(--text-dim);
}

/* Responsive */
@media (max-width: 900px) {
  .stats-row { grid-template-columns: repeat(2, 1fr); }
  .agent-grid { grid-template-columns: repeat(2, 1fr); }
  .bottom-grid { grid-template-columns: 1fr; }
}
</style>
