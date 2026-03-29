<script setup lang="ts">
/**
 * ChannelAggregator.vue - 通道聚合器
 * 显示 WeChat / Telegram / Discord 连接状态
 */
interface Channel {
  id: string
  name: string
  icon: string
  iconClass: string
  connected: boolean
  todayMessages: number
  lastActivity: string
}

import { ref, computed } from 'vue'

const channels = ref<Channel[]>([
  {
    id: 'wechat',
    name: 'WeChat',
    icon: '💬',
    iconClass: 'wechat',
    connected: true,
    todayMessages: 3,
    lastActivity: '14:32',
  },
  {
    id: 'telegram',
    name: 'Telegram',
    icon: '✈️',
    iconClass: 'telegram',
    connected: true,
    todayMessages: 24,
    lastActivity: '14:50',
  },
  {
    id: 'discord',
    name: 'Discord',
    icon: '🎮',
    iconClass: 'discord',
    connected: false,
    todayMessages: 0,
    lastActivity: '—',
  },
])

const activeCount = computed(() => channels.value.filter(c => c.connected).length)
</script>

<template>
  <div class="channel-agg">
    <div class="channel-agg-header">
      <h3>📡 通道聚合器</h3>
      <span class="channel-count">{{ activeCount }} / {{ channels.length }} 已连接</span>
    </div>

    <div class="channel-list">
      <div
        v-for="ch in channels"
        :key="ch.id"
        class="channel-item"
        :class="{ connected: ch.connected }"
      >
        <div class="ch-icon" :class="ch.iconClass">
          {{ ch.icon }}
        </div>
        <div class="ch-info">
          <div class="ch-name">
            {{ ch.name }}
            <span class="ch-status-dot" :class="{ online: ch.connected }"></span>
          </div>
          <div class="ch-stat">
            <template v-if="ch.connected">
              ✓ 已连接 · 今日 {{ ch.todayMessages }} 条消息
            </template>
            <template v-else>
              ✗ 未连接
            </template>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.channel-agg {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 16px;
}

.channel-agg-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 14px;
}

.channel-agg-header h3 {
  font-size: 13px;
  font-weight: 600;
}

.channel-count {
  font-size: 11px;
  color: var(--text-dim);
  font-family: var(--font-mono);
}

.channel-list {
  display: flex;
  gap: 12px;
}

.channel-item {
  flex: 1;
  background: var(--bg-base);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  padding: 14px;
  display: flex;
  align-items: center;
  gap: 12px;
  transition: border-color 0.2s;
  cursor: pointer;
  opacity: 0.7;
}

.channel-item.connected {
  opacity: 1;
}

.channel-item:hover {
  border-color: var(--border-light);
}

.channel-item.connected:hover {
  border-color: var(--cyan-dim);
}

.ch-icon {
  width: 36px;
  height: 36px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 18px;
  flex-shrink: 0;
}

.ch-icon.wechat  { background: rgba(34, 197,  94, 0.15); }
.ch-icon.telegram { background: rgba(59, 130, 246, 0.15); }
.ch-icon.discord  { background: rgba(139, 92, 246, 0.15); }

.ch-info {
  flex: 1;
  min-width: 0;
}

.ch-name {
  font-size: 12px;
  font-weight: 600;
  display: flex;
  align-items: center;
  gap: 6px;
}

.ch-status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--text-dim);
}

.ch-status-dot.online {
  background: var(--green);
  box-shadow: 0 0 4px var(--green);
}

.ch-stat {
  font-size: 10px;
  color: var(--text-dim);
  margin-top: 2px;
}
</style>
