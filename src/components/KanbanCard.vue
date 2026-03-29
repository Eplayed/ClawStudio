<script setup lang="ts">
/**
 * KanbanCard.vue - 单张 Agent 任务卡片
 */
import type { Agent } from '@/stores/agents'

const props = defineProps<{
  agent: Agent
}>()

const emit = defineEmits<{
  click: [agent: Agent]
}>()

const statusLabel: Record<string, string> = {
  queued: '队列中',
  running: '运行中',
  paused: '暂停',
  stopped: '已停止',
  completed: '已完成',
}

const statusClass: Record<string, string> = {
  queued: 'queued',
  running: 'running',
  paused: 'paused',
  stopped: 'stopped',
  completed: 'completed',
}

function formatTime(sec: number): string {
  if (sec < 60) return `${sec}s`
  if (sec < 3600) return `${Math.floor(sec / 60)}m`
  return `${Math.floor(sec / 3600)}h ${Math.floor((sec % 3600) / 60)}m`
}

function formatCost(cost: number): string {
  if (cost < 0.01) return '$' + cost.toFixed(4)
  return '$' + cost.toFixed(2)
}
</script>

<template>
  <div
    class="kanban-card"
    :class="{ 'kc-running': agent.status === 'running', 'kc-completed': agent.status === 'completed' }"
    @click="emit('click', agent)"
  >
    <div class="kc-header">
      <span class="kc-avatar">{{ agent.avatar }}</span>
      <div class="kc-info">
        <div class="kc-name">{{ agent.name }}</div>
        <div class="kc-id">{{ agent.id.toUpperCase() }}</div>
      </div>
    </div>

    <div class="kc-desc" v-if="agent.systemPrompt">
      {{ agent.systemPrompt.length > 60 ? agent.systemPrompt.slice(0, 60) + '…' : agent.systemPrompt }}
    </div>

    <div class="kc-meta">
      <span class="kc-cost">{{ formatCost(agent.currentCost) }}</span>
      <span class="kc-sep">·</span>
      <span class="kc-time">{{ formatTime(agent.elapsedSec) }}</span>
      <template v-if="agent.progress > 0">
        <span class="kc-sep">·</span>
        <span class="kc-progress">{{ agent.progress }}%</span>
      </template>
    </div>

    <!-- Progress bar for running agents -->
    <div class="kc-progress-bar" v-if="agent.status === 'running' && agent.progress > 0">
      <div class="kc-progress-fill" :style="{ width: agent.progress + '%' }"></div>
    </div>
  </div>
</template>

<style scoped>
.kanban-card {
  background: var(--bg-base);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  padding: 12px;
  margin-bottom: 8px;
  cursor: pointer;
  transition: all 0.2s;
  border-left: 2px solid transparent;
}

.kanban-card:hover {
  border-color: var(--border-light);
  transform: translateY(-1px);
  background: var(--bg-card-hover);
}

.kanban-card.kc-running {
  border-left-color: var(--green);
}

.kanban-card.kc-completed {
  opacity: 0.6;
  border-left-color: var(--cyan-dim);
}

.kc-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.kc-avatar {
  font-size: 18px;
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-elevated);
  border-radius: 6px;
  flex-shrink: 0;
}

.kc-info {
  flex: 1;
  min-width: 0;
}

.kc-name {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.kc-id {
  font-size: 10px;
  color: var(--text-dim);
  font-family: var(--font-mono);
}

.kc-desc {
  font-size: 11px;
  color: var(--text-dim);
  margin-bottom: 8px;
  line-height: 1.4;
}

.kc-meta {
  display: flex;
  gap: 6px;
  font-size: 10px;
  color: var(--text-dim);
  font-family: var(--font-mono);
  align-items: center;
}

.kc-sep {
  opacity: 0.4;
}

.kc-cost {
  color: var(--amber);
}

.kc-progress {
  color: var(--cyan);
}

.kc-progress-bar {
  height: 3px;
  background: var(--border);
  border-radius: 2px;
  margin-top: 8px;
  overflow: hidden;
}

.kc-progress-fill {
  height: 100%;
  border-radius: 2px;
  background: linear-gradient(90deg, var(--cyan), var(--green));
  transition: width 1s;
}
</style>
