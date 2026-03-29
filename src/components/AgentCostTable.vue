<script setup lang="ts">
/**
 * AgentCostTable.vue - Agent 费用明细表
 */
import { computed } from 'vue'
import { useAgentStore } from '@/stores/agents'
import { formatCost } from '@/utils/costCalculator'

const agentStore = useAgentStore()

const agents = computed(() => agentStore.agents)
const totalCost = computed(() => agentStore.totalCostToday)

const sortedAgents = computed(() => {
  return [...agents.value].sort((a, b) => b.currentCost - a.currentCost)
})

function getPercent(cost: number): number {
  if (totalCost.value <= 0) return 0
  return (cost / totalCost.value) * 100
}
</script>

<template>
  <div class="agent-cost-table">
    <h3 class="section-title">📋 Agent 费用明细</h3>
    
    <div v-if="sortedAgents.length === 0" class="empty-state">
      暂无活跃特工
    </div>
    
    <table v-else class="cost-table">
      <thead>
        <tr>
          <th>特工</th>
          <th>任务</th>
          <th>Tokens</th>
          <th>消耗</th>
          <th>占比</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="agent in sortedAgents" :key="agent.id">
          <td>
            <div class="agent-info">
              <span class="agent-icon">{{ agent.avatar || '🤖' }}</span>
              <span class="agent-name">{{ agent.name }}</span>
            </div>
          </td>
          <td class="task-cell">{{ agent.status }}</td>
          <td class="mono">{{ (agent.tokensUsed || 0).toLocaleString() }}</td>
          <td class="cost-cell" :class="{ highlight: agent.currentCost > 1 }">
            {{ formatCost(agent.currentCost) }}
          </td>
          <td>
            <div class="pct-cell">
              <div class="mini-bar">
                <div class="mini-fill" :style="{ width: getPercent(agent.currentCost) + '%' }"></div>
              </div>
              <span class="pct-text">{{ getPercent(agent.currentCost).toFixed(1) }}%</span>
            </div>
          </td>
        </tr>
      </tbody>
    </table>

    <div class="table-footer">
      <span>总计</span>
      <span class="footer-cost">{{ formatCost(totalCost) }}</span>
    </div>
  </div>
</template>

<style scoped>
.agent-cost-table {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 18px;
  margin-top: 14px;
}

.section-title {
  font-size: 13px;
  font-weight: 600;
  margin-bottom: 14px;
}

.empty-state {
  text-align: center;
  padding: 30px;
  color: var(--text-dim);
  font-size: 12px;
}

.cost-table {
  width: 100%;
  border-collapse: collapse;
}

.cost-table th {
  text-align: left;
  font-size: 10px;
  font-weight: 600;
  color: var(--text-dim);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  padding: 8px 10px;
  border-bottom: 1px solid var(--border);
}

.cost-table td {
  padding: 10px;
  border-bottom: 1px solid var(--border);
  font-size: 12px;
}

.agent-info {
  display: flex;
  align-items: center;
  gap: 8px;
}

.agent-icon {
  font-size: 14px;
}

.agent-name {
  font-weight: 500;
}

.task-cell {
  color: var(--text-secondary);
}

.mono {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--text-dim);
}

.cost-cell {
  font-family: var(--font-mono);
  font-weight: 600;
  color: var(--cyan);
}

.cost-cell.highlight {
  color: var(--amber);
}

.pct-cell {
  display: flex;
  align-items: center;
  gap: 8px;
}

.mini-bar {
  width: 50px;
  height: 4px;
  background: var(--border);
  border-radius: 2px;
  overflow: hidden;
}

.mini-fill {
  height: 100%;
  background: var(--cyan);
  border-radius: 2px;
}

.pct-text {
  font-size: 10px;
  font-family: var(--font-mono);
  color: var(--text-dim);
}

.table-footer {
  margin-top: 14px;
  padding-top: 12px;
  border-top: 1px solid var(--border);
  display: flex;
  justify-content: space-between;
  font-size: 12px;
  font-weight: 600;
}

.footer-cost {
  font-family: var(--font-mono);
  color: var(--cyan);
}
</style>
