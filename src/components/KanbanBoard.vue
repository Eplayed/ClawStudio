<script setup lang="ts">
/**
 * KanbanBoard.vue - 四列任务看板
 */
import { computed } from 'vue'
import KanbanCard from './KanbanCard.vue'
import type { Agent } from '@/stores/agents'

const props = defineProps<{
  agents: Agent[]
}>()

const emit = defineEmits<{
  'card-click': [agent: Agent]
}>()

const columns = [
  { key: 'queued',    label: '队列中', icon: '⏳', color: 'var(--text-secondary)' },
  { key: 'running',   label: '运行中', icon: '▶',  color: 'var(--green)' },
  { key: 'paused',    label: '暂停',   icon: '⏸',  color: 'var(--amber)' },
  { key: 'completed', label: '已完成', icon: '✓',  color: 'var(--text-dim)' },
]

const agentsByStatus = computed(() => {
  const map: Record<string, Agent[]> = {
    queued: [], running: [], paused: [], completed: [], stopped: [],
  }
  for (const agent of props.agents) {
    const key = agent.status === 'stopped' ? 'paused' : agent.status
    if (key in map) map[key].push(agent)
    else map.queued.push(agent)
  }
  return map
})
</script>

<template>
  <div class="kanban-board">
    <div
      v-for="col in columns"
      :key="col.key"
      class="kanban-col"
    >
      <div class="kanban-col-header">
        <span class="col-title" :style="{ color: col.color }">
          <span>{{ col.icon }}</span>
          {{ col.label }}
        </span>
        <span class="col-count">{{ agentsByStatus[col.key].length }}</span>
      </div>

      <KanbanCard
        v-for="agent in agentsByStatus[col.key]"
        :key="agent.id"
        :agent="agent"
        @click="emit('card-click', agent)"
      />

      <div v-if="agentsByStatus[col.key].length === 0" class="col-empty">
        <span>—</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.kanban-board {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 14px;
  margin-bottom: 24px;
}

.kanban-col {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 14px;
  min-height: 300px;
  display: flex;
  flex-direction: column;
}

.kanban-col-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 14px;
  padding-bottom: 10px;
  border-bottom: 1px solid var(--border);
}

.col-title {
  font-size: 12px;
  font-weight: 600;
  display: flex;
  align-items: center;
  gap: 6px;
}

.col-count {
  font-size: 10px;
  background: var(--bg-elevated);
  padding: 2px 8px;
  border-radius: 10px;
  color: var(--text-dim);
  font-family: var(--font-mono);
}

.col-empty {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-dim);
  font-size: 18px;
}
</style>
