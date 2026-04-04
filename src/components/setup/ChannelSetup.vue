<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const emit = defineEmits(['complete', 'next', 'back'])

interface Channel {
  id: string
  name: string
  icon: string
  connected: boolean
  description: string
}

interface ChannelConfig {
  channelType: string
  botToken?: string
  apiKey?: string
  channelId?: string
}

const channels = ref<Channel[]>([
  { id: 'telegram', name: 'Telegram', icon: '✈️', connected: false, description: '通过 Bot 与 Agent 交互' },
  { id: 'discord', name: 'Discord', icon: '🎮', connected: false, description: '在 Discord 服务器中使用 Agent' },
  { id: 'wechat', name: 'WeChat', icon: '💬', connected: false, description: '企业微信扫码连接' },
  { id: 'slack', name: 'Slack', icon: '💼', connected: false, description: '在 Slack 工作区中使用' },
])

const showConfigModal = ref(false)
const configModalChannel = ref<Channel | null>(null)
const configForm = ref<ChannelConfig>({
  channelType: '',
  botToken: '',
  apiKey: '',
  channelId: ''
})

const testingConnection = ref(false)
const testingChannelId = ref<string | null>(null)

onMounted(async () => {
  try {
    const result = await invoke<Channel[]>('list_channels')
    if (result && Array.isArray(result)) {
      channels.value.forEach(channel => {
        const matched = result.find((c: any) => c.channel_type === channel.id || c.id === channel.id)
        if (matched) {
          channel.connected = true
        }
      })
    }
  } catch (e) {
    console.error('Failed to list channels:', e)
  }
})

function openConfigModal(channel: Channel) {
  configModalChannel.value = channel
  configForm.value = {
    channelType: channel.id,
    botToken: '',
    apiKey: '',
    channelId: ''
  }
  showConfigModal.value = true
}

function closeConfigModal() {
  showConfigModal.value = false
  configModalChannel.value = null
}

async function connectChannel() {
  if (!configModalChannel.value) return

  try {
    await invoke('add_channel', {
      config: {
        channel_type: configForm.value.channelType,
        bot_token: configForm.value.botToken || null,
        api_key: configForm.value.apiKey || null,
        channel_id: configForm.value.channelId || null
      }
    })

    const channel = channels.value.find(c => c.id === configModalChannel.value!.id)
    if (channel) {
      channel.connected = true
    }

    closeConfigModal()
  } catch (e) {
    alert(`连接失败: ${e}`)
  }
}

async function disconnectChannel(channelId: string) {
  try {
    await invoke('remove_channel', { channelId })

    const channel = channels.value.find(c => c.id === channelId)
    if (channel) {
      channel.connected = false
    }
  } catch (e) {
    alert(`断开失败: ${e}`)
  }
}

async function testConnection(channel: Channel) {
  testingConnection.value = true
  testingChannelId.value = channel.id

  try {
    const result = await invoke<{ success: boolean; message: string }>('test_channel', {
      channelType: channel.id
    })

    if (result.success) {
      alert(`✅ ${channel.name} 连接正常!`)
    } else {
      alert(`❌ ${channel.name} 连接失败: ${result.message}`)
    }
  } catch (e) {
    alert(`❌ 测试失败: ${e}`)
  } finally {
    testingConnection.value = false
    testingChannelId.value = null
  }
}

function handleChannelClick(channel: Channel) {
  if (channel.connected) {
    disconnectChannel(channel.id)
  } else {
    openConfigModal(channel)
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
      >
        <div class="channel-icon">{{ channel.icon }}</div>
        <div class="channel-info">
          <h3>{{ channel.name }}</h3>
          <p>{{ channel.description }}</p>
        </div>
        <div class="channel-actions">
          <button
            class="btn-test"
            :disabled="!channel.connected || (testingConnection && testingChannelId === channel.id)"
            @click.stop="testConnection(channel)"
          >
            {{ testingConnection && testingChannelId === channel.id ? '测试中...' : '测试' }}
          </button>
          <button
            class="btn-toggle"
            :class="{ disconnect: channel.connected }"
            @click.stop="handleChannelClick(channel)"
          >
            {{ channel.connected ? '断开' : '连接' }}
          </button>
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

    <!-- Config Modal -->
    <div v-if="showConfigModal" class="modal-overlay" @click="closeConfigModal">
      <div class="modal" @click.stop>
        <h3>配置 {{ configModalChannel?.name }}</h3>

        <div v-if="configModalChannel?.id === 'wechat'" class="wechat-note">
          <p>⚠️ WeChat 需要在 OpenClaw 配置中手动设置</p>
          <p class="hint">请在 OpenClaw 配置文件中添加 WeChat 通道配置</p>
        </div>

        <div v-else class="config-form">
          <div v-if="configModalChannel?.id === 'telegram'" class="input-group">
            <label>Bot Token</label>
            <input
              type="password"
              v-model="configForm.botToken"
              placeholder="1234567890:ABCdefGHIjklMNOpqrsTUVwxyz"
            >
            <p class="hint">获取方式: @BotFather → /newbot</p>
          </div>

          <div v-if="configModalChannel?.id === 'discord'" class="input-group">
            <label>Bot Token</label>
            <input
              type="password"
              v-model="configForm.botToken"
              placeholder="Discord Bot Token"
            >
          </div>

          <div v-if="configModalChannel?.id === 'discord'" class="input-group">
            <label>Channel ID</label>
            <input
              type="text"
              v-model="configForm.channelId"
              placeholder="123456789012345678"
            >
            <p class="hint">在 Discord 中启用开发者模式，右键频道 → 复制 ID</p>
          </div>

          <div v-if="configModalChannel?.id === 'slack'" class="input-group">
            <label>Bot Token</label>
            <input
              type="password"
              v-model="configForm.botToken"
              placeholder="xoxb-..."
            >
          </div>

          <div v-if="configModalChannel?.id === 'slack'" class="input-group">
            <label>Channel ID</label>
            <input
              type="text"
              v-model="configForm.channelId"
              placeholder="C012345678"
            >
          </div>
        </div>

        <div class="modal-actions">
          <button class="btn btn-secondary" @click="closeConfigModal">取消</button>
          <button
            v-if="configModalChannel?.id !== 'wechat'"
            class="btn btn-primary"
            @click="connectChannel"
          >
            连接
          </button>
        </div>
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
  transition: all 0.2s;
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

.channel-actions {
  display: flex;
  gap: 8px;
}

.btn-test {
  padding: 6px 12px;
  font-size: 12px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.2s;
}

.btn-test:hover:not(:disabled) {
  border-color: var(--cyan);
  color: var(--cyan);
}

.btn-test:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-toggle {
  padding: 6px 16px;
  font-size: 12px;
  border: none;
  border-radius: 6px;
  background: var(--green);
  color: white;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-toggle.disconnect {
  background: var(--red);
}

.btn-toggle:hover {
  opacity: 0.9;
}

.channel-status {
  font-size: 13px;
  color: var(--text-secondary);
  min-width: 70px;
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

/* Modal Styles */
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal {
  background: var(--bg-deep);
  border: 1px solid var(--border-color);
  border-radius: 16px;
  padding: 24px;
  width: 400px;
  max-width: 90%;
}

.modal h3 {
  margin-bottom: 20px;
  font-size: 18px;
}

.wechat-note {
  text-align: center;
  padding: 20px;
}

.wechat-note p {
  margin-bottom: 8px;
}

.wechat-note .hint {
  font-size: 12px;
  color: var(--text-secondary);
}

.config-form {
  margin-bottom: 20px;
}

.input-group {
  margin-bottom: 16px;
}

.input-group label {
  display: block;
  margin-bottom: 6px;
  font-size: 14px;
  font-weight: 500;
}

.input-group input {
  width: 100%;
  padding: 10px 14px;
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  color: var(--text-primary);
  font-size: 14px;
}

.input-group input:focus {
  outline: none;
  border-color: var(--cyan);
}

.input-group .hint {
  margin-top: 6px;
  font-size: 12px;
  color: var(--text-secondary);
}

.modal-actions {
  display: flex;
  gap: 12px;
  justify-content: flex-end;
}
</style>