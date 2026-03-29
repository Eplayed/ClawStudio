<script setup lang="ts">
import { ref } from 'vue'
import { useSettingsStore } from '@/stores/settings'

const settings = useSettingsStore()

const claudeKey = ref('')
const openaiKey = ref('')
const testingClaude = ref(false)
const testingOpenai = ref(false)

async function handleTestKey(provider: 'claude' | 'openai') {
  const key = provider === 'claude' ? claudeKey.value : openaiKey.value
  if (provider === 'claude') testingClaude.value = true
  else testingOpenai.value = true

  try {
    const result = await settings.testKey(provider, key)
    if (result.valid) {
      await settings.saveKey(provider, key)
    }
  } catch (e) {
    console.error('Key test failed:', e)
  }

  if (provider === 'claude') testingClaude.value = false
  else testingOpenai.value = false
}

async function saveAll() {
  try {
    await settings.saveAllSettings()
    // Visual feedback handled by template
  } catch (e) {
    console.error('Failed to save settings:', e)
  }
}
</script>

<template>
  <div class="page">
    <header class="topbar">
      <div class="page-title">⚙ 系统设置 Settings</div>
    </header>
    <div class="content">
      <div class="settings-grid">
        <!-- API Keys -->
        <div class="settings-card">
          <h3>🔑 密钥保险箱</h3>
          <p class="desc">密钥通过操作系统原生 Keychain 加密存储</p>
          <div class="form-group">
            <label>Claude API Key</label>
            <div class="key-row">
              <input type="password" v-model="claudeKey" placeholder="sk-ant-..." />
              <button @click="handleTestKey('claude')" :disabled="testingClaude">
                {{ testingClaude ? '⏳...' : '测试' }}
              </button>
            </div>
            <div v-if="settings.claudeKeyValid" class="key-ok">✅ 已验证</div>
          </div>
          <div class="form-group">
            <label>OpenAI API Key (可选)</label>
            <div class="key-row">
              <input type="password" v-model="openaiKey" placeholder="sk-..." />
              <button @click="handleTestKey('openai')" :disabled="testingOpenai">
                {{ testingOpenai ? '⏳...' : '测试' }}
              </button>
            </div>
            <div v-if="settings.openaiKeyValid" class="key-ok">✅ 已验证</div>
          </div>
        </div>

        <!-- Model Prefs -->
        <div class="settings-card">
          <h3>🧠 模型偏好</h3>
          <p class="desc">为不同场景选择默认模型</p>
          <div class="form-group">
            <label>默认对话模型</label>
            <select v-model="settings.defaultModel">
              <option value="claude-3-5-sonnet-20241022">Claude 3.5 Sonnet</option>
              <option value="claude-3-opus-20240229">Claude 3 Opus</option>
              <option value="gpt-4o">GPT-4o</option>
            </select>
          </div>
          <div class="form-group">
            <label>Temperature: {{ settings.temperature.toFixed(2) }}</label>
            <input type="range" min="0" max="1" step="0.01" v-model.number="settings.temperature" />
          </div>
          <div class="form-group">
            <label>Max Tokens: {{ settings.maxTokens }}</label>
            <input type="range" min="256" max="8192" step="256" v-model.number="settings.maxTokens" />
          </div>
        </div>

        <!-- System Prompt -->
        <div class="settings-card full">
          <h3>📝 全局系统提示词</h3>
          <textarea v-model="settings.globalSystemPrompt" rows="4"></textarea>
        </div>

        <!-- Safety -->
        <div class="settings-card">
          <h3>🛡 安全与预算</h3>
          <div class="form-group">
            <label>单次任务预算上限: ${{ settings.budgetDefault.toFixed(2) }}</label>
            <input type="range" min="0.1" max="10" step="0.1" v-model.number="settings.budgetDefault" />
          </div>
          <div class="toggle-row">
            <span>HITL 审批系统</span>
            <input type="checkbox" v-model="settings.hitlEnabled" />
          </div>
          <div class="toggle-row">
            <span>删除操作拦截</span>
            <input type="checkbox" v-model="settings.interceptDelete" />
          </div>
          <div class="toggle-row">
            <span>发送/提交拦截</span>
            <input type="checkbox" v-model="settings.interceptSend" />
          </div>
          <div class="toggle-row">
            <span>Enter 键拦截</span>
            <input type="checkbox" v-model="settings.interceptEnter" />
          </div>
        </div>

        <!-- Network -->
        <div class="settings-card">
          <h3>🌐 本地与网络</h3>
          <div class="form-group">
            <label>HTTP 代理</label>
            <input type="text" v-model="settings.httpProxy" placeholder="http://127.0.0.1:7890" />
          </div>
          <div class="toggle-row">
            <span>Agent 网络访问</span>
            <input type="checkbox" v-model="settings.networkAccess" />
          </div>
          <div class="toggle-row">
            <span>零遥测模式</span>
            <input type="checkbox" v-model="settings.zeroTelemetry" />
          </div>
          <div class="form-group">
            <label>文件读写白名单</label>
            <input type="text" v-model="settings.fileWhitelist" />
          </div>
          <button class="btn-save" @click="saveAll">💾 保存全部设置</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.page { flex:1; display:flex; flex-direction:column; overflow:hidden; }
.topbar { height:52px; min-height:52px; display:flex; align-items:center; padding:0 24px; border-bottom:1px solid var(--border); background:var(--bg-base); }
.page-title { font-size:15px; font-weight:600; }
.content { flex:1; overflow-y:auto; padding:20px 24px; }

.settings-grid { display:grid; grid-template-columns:1fr 1fr; gap:16px; }
.settings-card {
  background:var(--bg-card); border:1px solid var(--border); border-radius:var(--radius); padding:20px;
}
.settings-card.full { grid-column: 1 / -1; }
.settings-card h3 { font-size:13px; font-weight:600; margin-bottom:6px; }
.desc { font-size:11px; color:var(--text-dim); margin-bottom:16px; }

.form-group { margin-bottom:14px; }
.form-group label { display:block; font-size:11px; font-weight:600; color:var(--text-secondary); margin-bottom:6px; }
.form-group input[type="text"],
.form-group input[type="password"],
.form-group select {
  width:100%; padding:9px 12px; background:var(--bg-base); border:1px solid var(--border);
  border-radius:var(--radius-sm); color:var(--text-primary); font-family:var(--font-mono);
  font-size:12px; outline:none;
}
.form-group input:focus, .form-group select:focus { border-color:var(--cyan-dim); }
.form-group textarea {
  width:100%; padding:10px 12px; background:var(--bg-base); border:1px solid var(--border);
  border-radius:var(--radius-sm); color:var(--text-primary); font-family:var(--font-mono);
  font-size:12px; resize:vertical; min-height:80px; outline:none;
}
.form-group input[type="range"] { width:100%; }

.key-row { display:flex; gap:8px; }
.key-row input { flex:1; }
.key-row button {
  padding:9px 14px; border-radius:var(--radius-sm); border:1px solid var(--border);
  background:var(--bg-elevated); color:var(--text-secondary); font-size:11px; cursor:pointer;
}
.key-ok { font-size:10px; color:var(--green); margin-top:4px; }

.toggle-row {
  display:flex; justify-content:space-between; align-items:center;
  padding:10px 0; border-bottom:1px solid var(--border); font-size:12px;
}
.toggle-row:last-of-type { border-bottom:none; }

.btn-save {
  padding:10px 28px; background:linear-gradient(135deg,var(--cyan),#0891b2);
  border:none; border-radius:var(--radius-sm); color:#fff; font-size:13px;
  font-weight:600; cursor:pointer; margin-top:12px; box-shadow:0 0 20px var(--cyan-glow);
}
</style>
