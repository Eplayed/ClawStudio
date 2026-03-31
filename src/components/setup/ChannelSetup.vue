<script setup lang="ts">
import { ref } from 'vue'

const emit = defineEmits(['complete', 'next', 'back'])

interface Channel {
  id: string
  name: string
  icon: string
  connected: boolean
  description: string
}

const channels = ref<Channel[]>([
  { id: 'telegram', name: 'Telegram', icon: '✈️', connected: false, description: '通过 Bot 与 Agent 交互' },
  { id: 'discord', name: 'Discord', icon: '🎮', connected: false, description: '在 Discord 服务器中使用 Agent' },
  { id: 'wechat', name: 'WeChat', icon: '💬', connected: false, description: '企业微信扫码连接' },
  { id: 'slack', name: 'Slack', icon: '💼', connected: false, description: '在 Slack 工作区中使用' },
])

function toggleChannel(channelId: string) {
  const channel = channels.value.find(c => c.id === channelId)
  if (channel) {
    channel.connected = !channel.connected
  }
}

function proceed() {
  const connectedChannels = channels.value
    .filter(c => c.connected)
    .map(c => c.id)
  
  emit('complete', { 
    channelsConfigured: connectedChannels.length > 0,
    channels: connectedChannels
  })
  emit('next')
}

function skip() {
  emit('complete', { channelsConfigured: false })
  emit('next')
}
</script>

<template>
  <div class="channel-setup">
    <h2>📱 配置通道 (可选)</h2>
    <p class="subtitle">连接消息通道，随时随地与 Agent 交互</p>

    <div class="channel-grid">
      <div 
        v-for="channel in channels" 
        :key="channel.id"
        class="channel-card"
        :class="{ connected: channel.connected }"
        @click="toggleChannel(channel.id)"
      >
        <div class="channel-icon">{{ channel.icon }}</div>
        <div class="channel-info">
          <h3>{{ channel.name }}</h3>
          <p>{{ channel.description }}</p>
        </div>
        <div class="channel-status">
          {{ channel.connected ? '✓ 已连接' : '未连接' }}
        </div>
      </div>
    </div>

    <div class="info-note">
      💡 您可以随时在设置中配置通道
    </div>

    <div class="footer-actions">
      <button class="btn btn-secondary" @click="$emit('back')">
        ← 上一步
      </button>
      <div class="right-actions">
        <button class="btn btn-outline" @click="skip">
          跳过
        </button>
        <button class="btn btn-primary" @click="proceed">
          继续 →
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.channel-setup {
  max-width: 600px;
  margin: 0 auto;
}

h2 {
  font-size: 24px;
  margin-bottom: 8px;
}

.subtitle {
  color: var(--text-secondary);
  margin-bottom: 32px;
}

.channel-grid {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-bottom: 24px;
}

.channel-card {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 16px 20px;
  background: var(--bg-card);
  border: 2px solid transparent;
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.2s;
}

.channel-card:hover {
  border-color: var(--border-color);
}

.channel-card.connected {
  border-color: var(--green);
  background: rgba(34, 197, 94, 0.05);
}

.channel-icon {
  font-size: 32px;
}

.channel-info {
  flex: 1;
}

.channel-info h3 {
  font-size: 16px;
  margin-bottom: 4px;
}

.channel-info p {
  font-size: 13px;
  color: var(--text-secondary);
}

.channel-status {
  font-size: 13px;
  color: var(--text-secondary);
}

.channel-card.connected .channel-status {
  color: var(--green);
}

.info-note {
  text-align: center;
  font-size: 13px;
  color: var(--text-secondary);
  margin-bottom: 32px;
}

.footer-actions {
  display: flex;
  justify-content: space-between;
}

.right-actions {
  display: flex;
  gap: 12px;
}

.btn {
  padding: 12px 24px;
  border-radius: 8px;
  font-weight: 600;
  cursor: pointer;
  border: none;
  transition: all 0.2s;
}

.btn-primary {
  background: var(--cyan);
  color: var(--bg-deep);
}

.btn-secondary {
  background: var(--bg-card);
  color: var(--text-primary);
}

.btn-outline {
  background: transparent;
  border: 1px solid var(--border-color);
  color: var(--text-primary);
}
</style>
