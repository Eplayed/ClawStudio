// Pinia store - Global settings
// Manages API keys, model prefs, HITL config, persisted to SQLite

import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export type PermLevel = 'browse' | 'standard' | 'auto'

export const useSettingsStore = defineStore('settings', () => {
  // API Keys (stored in OS Keychain, not here)
  const claudeKeyValid = ref(false)
  const openaiKeyValid = ref(false)

  // Model prefs
  const defaultModel = ref('claude-3-5-sonnet-20241022')
  const cuModel = ref('claude-3-5-sonnet-20241022')
  const temperature = ref(0.2)
  const maxTokens = ref(4096)

  // System prompt
  const globalSystemPrompt = ref(
    '你是一个高效的数字助手。在执行任何可能造成不可逆影响的操作前，必须暂停并等待用户确认。'
  )

  // HITL
  const hitlEnabled = ref(true)
  const interceptDelete = ref(true)
  const interceptSend = ref(true)
  const interceptEnter = ref(false)
  const permLevel = ref<PermLevel>('standard')
  const budgetDefault = ref(1.0)

  // Network
  const httpProxy = ref('')
  const networkAccess = ref(true)
  const zeroTelemetry = ref(true)
  const fileWhitelist = ref('/home/user/workspace, /tmp/agent-output')

  // Run mode: local or docker
  const runMode = ref<'local' | 'docker'>('docker')

  // ─── Database reference (set by App.vue after Database.load()) ───
  let _db: any = null

  function setDatabase(db: any) {
    _db = db
  }

  // ─── Persist to SQLite ───

  async function saveSetting(key: string, value: string) {
    if (!_db) return
    await _db.execute(
      'INSERT OR REPLACE INTO settings (key, value, updated_at) VALUES (?, ?, datetime("now"))',
      [key, value]
    )
  }

  async function loadAllSettings() {
    if (!_db) return
    try {
      const rows: Array<{ key: string; value: string }> = await _db.select(
        'SELECT key, value FROM settings'
      )
      for (const row of rows) {
        switch (row.key) {
          case 'default_model': defaultModel.value = row.value; break
          case 'cu_model': cuModel.value = row.value; break
          case 'temperature': temperature.value = parseFloat(row.value); break
          case 'max_tokens': maxTokens.value = parseInt(row.value); break
          case 'hitl_enabled': hitlEnabled.value = row.value === 'true'; break
          case 'hitl_intercept_delete': interceptDelete.value = row.value === 'true'; break
          case 'hitl_intercept_send': interceptSend.value = row.value === 'true'; break
          case 'hitl_intercept_enter': interceptEnter.value = row.value === 'true'; break
          case 'perm_level': permLevel.value = row.value as PermLevel; break
          case 'budget_default': budgetDefault.value = parseFloat(row.value); break
          case 'zero_telemetry': zeroTelemetry.value = row.value === 'true'; break
          case 'http_proxy': httpProxy.value = row.value; break
          case 'network_access': networkAccess.value = row.value === 'true'; break
          case 'file_whitelist': fileWhitelist.value = row.value; break
          case 'global_system_prompt': globalSystemPrompt.value = row.value; break
          case 'run_mode': runMode.value = row.value as 'local' | 'docker'; break
        }
      }
    } catch (e) {
      console.warn('Failed to load settings from DB:', e)
    }
  }

  async function saveAllSettings() {
    const entries: [string, string][] = [
      ['default_model', defaultModel.value],
      ['cu_model', cuModel.value],
      ['temperature', temperature.value.toString()],
      ['max_tokens', maxTokens.value.toString()],
      ['hitl_enabled', hitlEnabled.value.toString()],
      ['hitl_intercept_delete', interceptDelete.value.toString()],
      ['hitl_intercept_send', interceptSend.value.toString()],
      ['hitl_intercept_enter', interceptEnter.value.toString()],
      ['perm_level', permLevel.value],
      ['budget_default', budgetDefault.value.toString()],
      ['zero_telemetry', zeroTelemetry.value.toString()],
      ['http_proxy', httpProxy.value],
      ['network_access', networkAccess.value.toString()],
      ['file_whitelist', fileWhitelist.value],
      ['global_system_prompt', globalSystemPrompt.value],
      ['run_mode', runMode.value],
    ]
    for (const [key, value] of entries) {
      await saveSetting(key, value)
    }
  }

  // ─── Keychain Operations ───

  async function saveKey(provider: string, key: string) {
    await invoke('save_api_key', { provider, key })
  }

  async function testKey(provider: string, key: string) {
    const result = await invoke<{
      valid: boolean
      model: string
      message: string
    }>('test_api_key', { provider, key })
    if (provider === 'claude') claudeKeyValid.value = result.valid
    if (provider === 'openai') openaiKeyValid.value = result.valid
    return result
  }

  async function loadKey(provider: string): Promise<string> {
    try {
      return await invoke<string>('get_api_key', { provider })
    } catch {
      return ''
    }
  }

  return {
    claudeKeyValid,
    openaiKeyValid,
    defaultModel,
    cuModel,
    temperature,
    maxTokens,
    globalSystemPrompt,
    hitlEnabled,
    interceptDelete,
    interceptSend,
    interceptEnter,
    permLevel,
    budgetDefault,
    httpProxy,
    networkAccess,
    zeroTelemetry,
    fileWhitelist,
    runMode,
    setDatabase,
    loadAllSettings,
    saveAllSettings,
    saveKey,
    testKey,
    loadKey,
  }
})
