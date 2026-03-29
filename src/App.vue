<script setup lang="ts">
import Sidebar from '@/components/Sidebar.vue'
import { useAgentStore } from '@/stores/agents'
import { useSettingsStore } from '@/stores/settings'
import { invoke } from '@tauri-apps/api/core'
import { onMounted } from 'vue'

const agentStore = useAgentStore()
const settingsStore = useSettingsStore()

interface QueryPlan {
  sql: string
  params: unknown[]
}

onMounted(async () => {
  // Initialize event listener for OpenClaw process events
  agentStore.initEventListener()

  // Initialize database
  try {
    // Dynamic import for plugin-sql (only available in Tauri context)
    const { default: Database } = await import('@tauri-apps/plugin-sql')
    const db = await Database.load('sqlite:clawstudio.db')

    // Run migrations
    const plans = await invoke<QueryPlan[]>('init_database')
    for (const plan of plans) {
      await db.execute(plan.sql, plan.params)
    }

    // Inject DB into settings store and load persisted values
    settingsStore.setDatabase(db)
    await settingsStore.loadAllSettings()
  } catch (e) {
    // Expected to fail in browser dev mode (no Tauri runtime)
    console.warn('Tauri DB init skipped (browser mode):', e)
  }
})
</script>

<template>
  <Sidebar />
  <main class="main-content">
    <router-view v-slot="{ Component }">
      <transition name="fade" mode="out-in">
        <component :is="Component" />
      </transition>
    </router-view>
  </main>
</template>

<style scoped>
.main-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--bg-deep);
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.15s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
