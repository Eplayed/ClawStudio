<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

interface ChannelInfo {
  id: string
  channel_type: string
  name: string
  connected: boolean
  config: any
}

const channels = ref<ChannelInfo[]>([])
const loading = ref(true)
const testing = ref<string | null>(null)
const testResults = ref<Record<string, { success: boolean; message: string }>>({})

// Modal state
const showAddModal = ref(false)
const addChannelType = ref('')
const botToken = ref('')

async function loadChannels() {
  loading.value = true
  try {
    channels.value = await invoke('list_channels')
  } catch (e) {
    console.error('Failed to load channels:', e)
  } finally {
    loading.value = false
  }
}

async function testChannel(channel: ChannelInfo) {
  testing.value = channel.id
  testResults.value[channel.id] = { success: false, message: '测试中...' }
  
  try {
    const result = await invoke('test_channel', {
      channelType: channel.channel_type,
      config: { bot_token: botToken.value }
    })
    testResults.value[channel.id] = result as any
  } catch (e) {
    testResults.value[channel.id] = { success: false, message: `测试失败: ${e}` }
  } finally {
    testing.value = null
  }
}

async function addChannel() {
  if (!addChannelType.value || !botToken.value) return
  
  try {
    await invoke('add_channel', {
      config: {
        channel_type: addChannelType.value,
        bot_token: botToken.value
      }
    })
    
    // Restart gateway
    await invoke('restart_gateway_for_channels')
    
    showAddModal.value = false
    botToken.value = ''
    addChannelType.value = ''
    await loadChannels()
  } catch (e) {
    console.error('Failed to add channel:', e)
  }
}

async function removeChannel(channel: ChannelInfo) {
  if (!confirm(`确定要断开 ${channel.name} 吗？`)) return
  
  try {
    await invoke('remove_channel', { channelType: channel.channel_type })
    await invoke('restart_gateway_for_channels')
    await loadChannels()
  } catch (e) {
    console.error('Failed to remove channel:', e)
  }
}

function openAddModal(type_: string) {
  addChannelType.value = type_
  showAddModal.value = true
}

onMounted(loadChannels)
</script>

<template>
  <div class="channel-manager">
    <div class="section-header">
      <h3>📱 通道管理</h3>
      <button class="btn btn-sm btn-primary" @click="showAddModal = true">
        + 添加通道
      </button>
    </div>

    <div v-if="loading" class="loading">加载中...</div>

    <div v-else class="channel-list">
      <div 
        v-for="channel in channels" 
        :key="channel.id"
        class="channel-card"
        :class="{ connected: channel.connected }"
      >
        <div class="channel-icon">
          {{ channel.channel_type === 'telegram' ? '✈️' : 
             channel.channel_type === 'discord' ? '🎮' :
             channel.channel_type === 'wechat' ? '💬' : '💼' }}
        </div>
        
        <div class="channel-info">
          <h4>{{ channel.name }}</h4>
          <p class="status">
            {{ channel.connected ? '✓ 已连接' : '○ 未连接' }}
          </p>
        </div>

        <div class="channel-actions">
          <button 
            v-if="channel.channel_type !== 'wechat'"
            class="btn btn-sm btn-outline"
            :disabled="testing === channel.id"
            @click="testChannel(channel)"
          >
            {{ testing === channel.id ? '测试中...' : '测试' }}
          </button>
          
          <button 
            v-if="channel.connected"
            class="btn btn-sm btn-danger"
            @click="removeChannel(channel)"
          >
            断开
          </button>
          
          <button 
            v-else
            class="btn btn-sm btn-primary"
            @click="openAddModal(channel.channel_type)"
          >
            连接
          </button>
        </div>

        <!-- Test Result -->
        <div v-if="testResults[channel.id]" class="test-result" :class="{ success: testResults[channel.id].success }">
          {{ testResults[channel.id].message }}
        </div>
      </div>
    </div>

    <!-- Add Channel Modal -->
    <div v-if="showAddModal" class="modal-overlay" @click.self="showAddModal = false">
      <div class="modal">
        <h3>添加 {{ addChannelType === 'telegram' ? 'Telegram' : 
                                addChannelType === 'discord' ? 'Discord' : 
                                addChannelType === 'slack' ? 'Slack' : '' }} 通道</h3>
        
        <div class="form-group">
          <label>Bot Token</label>
          <input 
            type="password" 
            v-model="botToken" 
            :placeholder="addChannelType === 'telegram' ? '1234567890:ABCdefGHIjklMNOpqrsTUVwxyz' : 'Discord Bot Token'"
          />
          <p class="hint">
            {{ addChannelType === 'telegram' 
              ? '从 @BotFather 获取 Bot Token' 
              : '从 Discord Developer Portal 获取 Bot Token' }}
          </p>
        </div>

        <div class="modal-actions">
          <button class="btn btn-secondary" @click="showAddModal = false">取消</button>
          <button class="btn btn-primary" @click="addChannel">添加</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.channel-manager {
  padding: 20px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.section-header h3 {
  margin: 0;
}

.channel-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.channel-card {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 16px;
  background: var(--bg-card);
  border-radius: 12px;
  border: 2px solid transparent;
}

.channel-card.connected {
  border-color: var(--green);
}

.channel-icon {
  font-size: 32px;
}

.channel-info {
  flex: 1;
}

.channel-info h4 {
  margin: 0 0 4px 0;
}

.status {
  font-size: 13px;
  color: var(--text-secondary);
  margin: 0;
}

.channel-actions {
  display: flex;
  gap: 8px;
}

.test-result {
  width: 100%;
  padding: 8px 12px;
  margin-top: 8px;
  border-radius: 6px;
  font-size: 13px;
  background: rgba(239, 68, 68, 0.1);
  color: var(--red);
}

.test-result.success {
  background: rgba(34, 197, 94, 0.1);
  color: var(--green);
}

.btn {
  padding: 8px 16px;
  border-radius: 6px;
  font-weight: 500;
  cursor: pointer;
  border: none;
  transition: all 0.2s;
}

.btn-sm {
  padding: 6px 12px;
  font-size: 13px;
}

.btn-primary {
  background: var(--cyan);
  color: var(--bg-deep);
}

.btn-secondary {
  background: var(--bg-base);
  color: var(--text-primary);
}

.btn-outline {
  background: transparent;
  border: 1px solid var(--border-color);
  color: var(--text-primary);
}

.btn-danger {
  background: var(--red);
  color: white;
}

/* Modal */
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
  background: var(--bg-base);
  border-radius: 16px;
  padding: 24px;
  width: 400px;
  max-width: 90%;
}

.modal h3 {
  margin: 0 0 20px 0;
}

.form-group {
  margin-bottom: 20px;
}

.form-group label {
  display: block;
  margin-bottom: 8px;
  font-weight: 500;
}

.form-group input {
  width: 100%;
  padding: 12px;
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  color: var(--text-primary);
}

.hint {
  margin-top: 8px;
  font-size: 12px;
  color: var(--text-secondary);
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}
</style>
