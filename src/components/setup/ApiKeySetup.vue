<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const emit = defineEmits(['complete', 'next', 'back'])

const provider = ref('claude')
const apiKey = ref('')
const testing = ref(false)
const testResult = ref<{ valid: boolean; message: string } | null>(null)
const saving = ref(false)

async function testKey() {
  if (!apiKey.value) return
  
  testing.value = true
  testResult.value = null

  try {
    const result = await invoke('test_api_key', { 
      provider: provider.value, 
      key: apiKey.value 
    })
    testResult.value = result as any
  } catch (e) {
    testResult.value = { valid: false, message: `测试失败: ${e}` }
  } finally {
    testing.value = false
  }
}

async function saveAndProceed() {
  saving.value = true
  
  try {
    // Save to keychain
    await invoke('save_api_key', { 
      provider: provider.value, 
      key: apiKey.value 
    })
    
    emit('complete', { 
      apiKeyConfigured: true,
      provider: provider.value 
    })
    emit('next')
  } catch (e) {
    testResult.value = { valid: false, message: `保存失败: ${e}` }
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <div class="api-key-setup">
    <h2>🔑 配置 API Key</h2>
    <p class="subtitle">输入您的 AI 服务 API Key</p>

    <!-- Provider Selection -->
    <div class="provider-tabs">
      <button 
        class="tab" 
        :class="{ active: provider === 'claude' }"
        @click="provider = 'claude'"
      >
        🟣 Anthropic Claude
      </button>
      <button 
        class="tab" 
        :class="{ active: provider === 'openai' }"
        @click="provider = 'openai'"
      >
        🟢 OpenAI
      </button>
    </div>

    <!-- API Key Input -->
    <div class="input-group">
      <label>{{ provider === 'claude' ? 'Anthropic' : 'OpenAI' }} API Key</label>
      <input 
        :type="provider === 'claude' ? 'password' : 'password'"
        v-model="apiKey"
        :placeholder="provider === 'claude' ? 'sk-ant-api03-...' : 'sk-...'"
      >
      <p class="hint">
        {{ provider === 'claude' 
          ? '获取 Key: https://console.anthropic.com/settings/keys' 
          : '获取 Key: https://platform.openai.com/api-keys' 
        }}
      </p>
    </div>

    <!-- Test Result -->
    <div v-if="testResult" class="test-result" :class="{ success: testResult.valid, error: !testResult.valid }">
      {{ testResult.valid ? '✅' : '❌' }} {{ testResult.message }}
    </div>

    <!-- Actions -->
    <div class="actions">
      <button 
        class="btn btn-secondary" 
        :disabled="!apiKey || testing"
        @click="testKey"
      >
        {{ testing ? '测试中...' : '测试连接' }}
      </button>
      <button 
        class="btn btn-primary" 
        :disabled="!apiKey || !testResult?.valid || saving"
        @click="saveAndProceed"
      >
        {{ saving ? '保存中...' : '保存并继续' }}
      </button>
    </div>

    <div class="footer-actions">
      <button class="btn btn-secondary" @click="$emit('back')">
        ← 上一步
      </button>
    </div>
  </div>
</template>

<style scoped>
.api-key-setup {
  max-width: 500px;
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

.provider-tabs {
  display: flex;
  gap: 8px;
  margin-bottom: 24px;
}

.tab {
  flex: 1;
  padding: 12px;
  background: var(--bg-card);
  border: 2px solid transparent;
  border-radius: 8px;
  cursor: pointer;
  font-weight: 500;
  transition: all 0.2s;
}

.tab.active {
  border-color: var(--cyan);
  background: rgba(6, 214, 214, 0.1);
}

.input-group {
  margin-bottom: 24px;
}

.input-group label {
  display: block;
  margin-bottom: 8px;
  font-weight: 500;
}

.input-group input {
  width: 100%;
  padding: 12px 16px;
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

.hint {
  margin-top: 8px;
  font-size: 12px;
  color: var(--text-secondary);
}

.test-result {
  padding: 12px;
  border-radius: 8px;
  margin-bottom: 24px;
  font-size: 14px;
}

.test-result.success {
  background: rgba(34, 197, 94, 0.1);
  border: 1px solid var(--green);
}

.test-result.error {
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid var(--red);
}

.actions {
  display: flex;
  gap: 12px;
  justify-content: center;
}

.footer-actions {
  display: flex;
  justify-content: flex-start;
  margin-top: 32px;
}

.btn {
  padding: 12px 24px;
  border-radius: 8px;
  font-weight: 600;
  cursor: pointer;
  border: none;
  transition: all 0.2s;
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-primary {
  background: var(--cyan);
  color: var(--bg-deep);
}

.btn-secondary {
  background: var(--bg-card);
  color: var(--text-primary);
}
</style>
