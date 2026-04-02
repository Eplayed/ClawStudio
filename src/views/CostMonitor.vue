<script setup lang="ts">
/**
 * CostMonitor.vue - 烧钱计算器页面
 * 预算油表 + Token 分解 + Agent 费用明细 + 自动熔断
 */
import { ref, computed } from 'vue'
import { useAgentStore } from '@/stores/agents'
import { useSettingsStore } from '@/stores/settings'
import { useProxyStore } from '@/stores/proxy'
import FuelGauge from '@/components/FuelGauge.vue'
import TokenBreakdown from '@/components/TokenBreakdown.vue'
import AgentCostTable from '@/components/AgentCostTable.vue'

const agentStore = useAgentStore()
const settingsStore = useSettingsStore()
const proxy = useProxyStore()

// Budget
const budgetLimit = computed(() => proxy.budgetLimit)
const currentCost = computed(() => proxy.totalCost)

// Token breakdown (mock for now, will connect to real data)
const inputTokens = computed(() => agentStore.agents.reduce((sum, a) => sum + (a.tokensUsed || 0) * 0.3, 125000))
const outputTokens = computed(() => agentStore.agents.reduce((sum, a) => sum + (a.tokensUsed || 0) * 0.5, 45000))
const imageTokens = computed(() => agentStore.agents.reduce((sum, a) => sum + (a.tokensUsed || 0) * 0.2, 32000))

// Auto cutoff slider
const cutoffValue = ref(settingsStore.budgetDefault)
const minCutoff = 0.5
const maxCutoff = 50
const stepCutoff = 0.5

function onCutoffChange() {
  settingsStore.budgetDefault = cutoffValue.value
  settingsStore.saveAllSettings()
}

// Mock agents if empty
const displayAgents = computed(() => {
  if (agentStore.agents.length === 0) {
    return [
      { id: 'agent-01', name: '整理发票', avatar: '📄', status: 'running', currentCost: 0.34, tokensUsed: 45000 },
      { id: 'agent-02', name: '监控竞品', avatar: '🔍', status: 'sleeping', currentCost: 0.12, tokensUsed: 12000 },
      { id: 'agent-03', name: '客服回复', avatar: '💬', status: 'running', currentCost: 1.89, tokensUsed: 98000 },
    ]
  }
  return agentStore.agents
})
</script>

<template>
  <div class="page">
    <header class="topbar">
      <div class="page-title">💰 烧钱计算器 Cost Monitor</div>
      <div style="display: flex; align-items: center; gap: 12px;">
        <button class="btn btn-outline" @click="proxy.resetCost()" style="font-size: 12px; padding: 4px 10px; border-radius: 4px; border: 1px solid var(--border); background: var(--bg-card); cursor: pointer; color: inherit;">重置当前计数</button>
        <div class="status-chip amber">
          <span class="icon">🔥</span>
          实时计费中
        </div>
      </div>
    </header>

    <div class="content">
      <div class="monitor-layout">
        <!-- Left: Fuel Gauge -->
        <div class="gauge-column">
          <div class="gauge-panel">
            <FuelGauge :current="currentCost" :limit="budgetLimit" />
          </div>
        </div>

        <!-- Right: Token Breakdown + Agent Table -->
        <div class="data-column">
          <TokenBreakdown
            :input-tokens="inputTokens"
            :output-tokens="outputTokens"
            :image-tokens="imageTokens"
          />
          <AgentCostTable />
        </div>
      </div>

      <!-- Auto Cutoff Section -->
      <div class="cutoff-section">
        <div class="cutoff-header">
          <h3>⚡ 自动熔断 Auto Cutoff</h3>
          <span class="cutoff-value">${{ cutoffValue.toFixed(2) }}</span>
        </div>
        <div class="cutoff-control">
          <input
            type="range"
            :min="minCutoff"
            :max="maxCutoff"
            :step="stepCutoff"
            v-model.number="cutoffValue"
            @input="onCutoffChange"
            class="cutoff-slider"
          />
          <div class="slider-labels">
            <span>$0.50</span>
            <span>$50.00</span>
          </div>
        </div>
        <p class="cutoff-desc">
          当任意 Agent 消耗超过此上限时自动暂停并弹出通知
        </p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.page {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
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
}

.page-title {
  font-size: 15px;
  font-weight: 600;
}

.status-chip {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 12px;
  border-radius: 20px;
  font-size: 11px;
  font-weight: 500;
}

.status-chip.amber {
  background: var(--amber-glow);
  color: var(--amber);
  border: 1px solid rgba(240, 160, 48, 0.2);
}

.content {
  flex: 1;
  overflow-y: auto;
  padding: 20px 24px;
}

.monitor-layout {
  display: grid;
  grid-template-columns: 280px 1fr;
  gap: 20px;
  margin-bottom: 24px;
}

.gauge-column {
  display: flex;
  flex-direction: column;
}

.gauge-panel {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 320px;
}

.data-column {
  display: flex;
  flex-direction: column;
}

/* Cutoff Section */
.cutoff-section {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 20px;
}

.cutoff-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 14px;
}

.cutoff-header h3 {
  font-size: 14px;
  font-weight: 600;
}

.cutoff-value {
  font-family: var(--font-mono);
  font-size: 16px;
  font-weight: 700;
  color: var(--amber);
  background: var(--amber-glow);
  padding: 4px 12px;
  border-radius: var(--radius-sm);
}

.cutoff-control {
  margin-bottom: 10px;
}

.cutoff-slider {
  width: 100%;
  height: 6px;
  -webkit-appearance: none;
  appearance: none;
  background: var(--border);
  border-radius: 3px;
  outline: none;
}

.cutoff-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 20px;
  height: 20px;
  border-radius: 50%;
  background: linear-gradient(135deg, var(--amber), #d97706);
  cursor: pointer;
  box-shadow: 0 0 10px var(--amber-glow);
}

.cutoff-slider::-moz-range-thumb {
  width: 20px;
  height: 20px;
  border-radius: 50%;
  background: linear-gradient(135deg, var(--amber), #d97706);
  cursor: pointer;
  border: none;
}

.slider-labels {
  display: flex;
  justify-content: space-between;
  margin-top: 6px;
  font-size: 10px;
  color: var(--text-dim);
  font-family: var(--font-mono);
}

.cutoff-desc {
  font-size: 11px;
  color: var(--text-dim);
  margin: 0;
}

/* Responsive */
@media (max-width: 900px) {
  .monitor-layout {
    grid-template-columns: 1fr;
  }
}
</style>
