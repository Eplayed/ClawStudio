<script setup lang="ts">
/**
 * SandboxCard.vue - 单个沙盒卡片
 * D6: CPU/Memory 实时 mini-charts
 */
interface Sandbox {
  id: string
  name: string
  image: string
  status: string
  vnc_port: number
}

interface SandboxStats {
  cpu_percent: number
  memory_used_mb: number
  memory_limit_mb: number
}

defineProps<{
  sandbox: Sandbox
  stats?: SandboxStats
}>()

defineEmits<{
  'connect-vnc': [sandbox: Sandbox]
  destroy: [id: string]
  start: [id: string]
  stop: [id: string]
}>()

function getResourceColor(percent: number): string {
  if (percent < 50) return 'var(--green)'
  if (percent < 80) return 'var(--amber)'
  return 'var(--red)'
}

function getMemoryPercent(used: number, limit: number): number {
  if (limit <= 0) return 0
  return (used / limit) * 100
}
</script>

<template>
  <div class="sandbox-card">
    <!-- Card Header -->
    <div class="card-top">
      <div class="sb-icon ubuntu">🐧</div>
      <div class="card-info">
        <div class="sb-name">{{ sandbox.name }}</div>
        <div class="sb-image">{{ sandbox.image }}</div>
      </div>
      <span class="status-badge" :class="sandbox.status">
        {{ sandbox.status === 'running' ? '运行中' : '已停止' }}
      </span>
    </div>

    <!-- D6: Resource Monitoring -->
    <div v-if="stats" class="resource-monitor">
      <!-- CPU -->
      <div class="resource-row">
        <div class="resource-label">CPU</div>
        <div class="resource-bar">
          <div
            class="resource-fill"
            :style="{
              width: stats.cpu_percent + '%',
              background: getResourceColor(stats.cpu_percent)
            }"
          ></div>
        </div>
        <span class="resource-value">{{ stats.cpu_percent.toFixed(1) }}%</span>
      </div>

      <!-- Memory -->
      <div class="resource-row">
        <div class="resource-label">MEM</div>
        <div class="resource-bar">
          <div
            class="resource-fill"
            :style="{
              width: getMemoryPercent(stats.memory_used_mb, stats.memory_limit_mb) + '%',
              background: getResourceColor(getMemoryPercent(stats.memory_used_mb, stats.memory_limit_mb))
            }"
          ></div>
        </div>
        <span class="resource-value">{{ stats.memory_used_mb }} / {{ stats.memory_limit_mb }} MB</span>
      </div>
    </div>

    <div v-else class="resource-monitor">
      <div class="resource-waiting">等待数据…</div>
    </div>

    <!-- Actions -->
    <div class="card-actions">
      <button
        v-if="sandbox.status === 'running'"
        class="action-btn vnc"
        @click="$emit('connect-vnc', sandbox)"
      >
        🖥 VNC
      </button>
      <button
        v-if="sandbox.status === 'running'"
        class="action-btn stop"
        @click="$emit('stop', sandbox.id)"
      >
        ⏸ 停止
      </button>
      <button
        v-else
        class="action-btn start"
        @click="$emit('start', sandbox.id)"
      >
        ▶ 启动
      </button>
      <button
        class="action-btn destroy"
        @click="$emit('destroy', sandbox.id)"
      >
        🗑 销毁
      </button>
    </div>
  </div>
</template>

<style scoped>
.sandbox-card {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 18px;
  transition: all 0.25s;
}

.sandbox-card:hover {
  border-color: var(--border-light);
}

.card-top {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 14px;
}

.sb-icon {
  width: 42px;
  height: 42px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 20px;
}

.sb-icon.ubuntu {
  background: rgba(233, 84, 32, 0.15);
}

.card-info {
  flex: 1;
  margin-left: 12px;
}

.sb-name {
  font-size: 13px;
  font-weight: 600;
  margin-bottom: 2px;
}

.sb-image {
  font-size: 10px;
  color: var(--text-dim);
  font-family: var(--font-mono);
}

.status-badge {
  font-size: 10px;
  padding: 3px 8px;
  border-radius: 10px;
  font-weight: 600;
}

.status-badge.running {
  background: var(--green-glow);
  color: var(--green);
}

.status-badge.stopped {
  background: var(--amber-glow);
  color: var(--amber);
}

/* D6: Resource Monitor */
.resource-monitor {
  background: var(--bg-base);
  border-radius: var(--radius-sm);
  padding: 12px;
  margin-bottom: 14px;
}

.resource-row {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 8px;
}

.resource-row:last-child {
  margin-bottom: 0;
}

.resource-label {
  font-size: 10px;
  font-weight: 600;
  color: var(--text-dim);
  min-width: 30px;
}

.resource-bar {
  flex: 1;
  height: 4px;
  background: var(--border);
  border-radius: 2px;
  overflow: hidden;
}

.resource-fill {
  height: 100%;
  border-radius: 2px;
  transition: width 0.5s cubic-bezier(0.4, 0, 0.2, 1);
}

.resource-value {
  font-size: 10px;
  font-family: var(--font-mono);
  color: var(--text-dim);
  min-width: 70px;
  text-align: right;
}

.resource-waiting {
  font-size: 11px;
  color: var(--text-dim);
  text-align: center;
  padding: 8px;
}

/* Actions */
.card-actions {
  display: flex;
  gap: 8px;
}

.action-btn {
  flex: 1;
  padding: 6px 10px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border);
  background: var(--bg-base);
  color: var(--text-secondary);
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
  font-family: var(--font-ui);
}

.action-btn:hover {
  border-color: var(--cyan-dim);
  color: var(--cyan);
}

.action-btn.vnc {
  border-color: var(--cyan-dim);
  color: var(--cyan);
  background: var(--cyan-glow);
}

.action-btn.vnc:hover {
  box-shadow: 0 0 10px var(--cyan-glow);
}

.action-btn.start {
  border-color: var(--green-dim);
  color: var(--green);
  background: var(--green-glow);
}

.action-btn.start:hover {
  box-shadow: 0 0 10px var(--green-glow);
}

.action-btn.destroy {
  border-color: var(--red-dim);
  color: var(--red);
}

.action-btn.destroy:hover {
  background: var(--red-glow);
  box-shadow: 0 0 10px var(--red-glow);
}
</style>
