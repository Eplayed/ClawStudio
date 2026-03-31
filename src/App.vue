<script setup lang="ts">
import Sidebar from '@/components/Sidebar.vue'
import GatewayStatusBar from '@/components/GatewayStatusBar.vue'
import { useAgentStore } from '@/stores/agents'
import { useSettingsStore } from '@/stores/settings'
import { invoke } from '@tauri-apps/api/core'
import { onMounted, ref, computed } from 'vue'
import { useRouter, useRoute } from 'vue-router'

const router = useRouter()
const route = useRoute()
const agentStore = useAgentStore()
const settingsStore = useSettingsStore()
const isReady = ref(false)

// Check if we're on a page that needs the sidebar
const _showSidebar = computed(() => {
  return route.name !== 'setup' && !route.meta?.fullscreen
})

interface QueryPlan {
  sql: string
  params: unknown[]
}

// Check if OpenClaw is installed and guide to setup if not
async function checkAndSetup() {
  try {
    const envStatus = await invoke<any>('get_env_status')
    const needsSetup = !envStatus?.openclaw?.installed || !envStatus?.gateway?.running
    
    // Redirect to setup if not fully configured and not already on setup page
    if (needsSetup && route.name !== 'setup') {
      router.replace('/setup')
    }
    isReady.value = true
  } catch (e) {
    // If we can't check, assume first run
    if (route.name !== 'setup') {
      router.replace('/setup')
    }
    isReady.value = true
  }
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
    
    // Check environment after DB is ready
    await checkAndSetup()
  } catch (e) {
    // Expected to fail in browser dev mode (no Tauri runtime)
    console.warn('Tauri DB init skipped (browser mode):', e)
    isReady.value = true
  }
})
</script>

<template>
  <!-- Setup Wizard (fullscreen, no sidebar) -->
  <template v-if="route.name === 'setup' || route.meta?.fullscreen">
    <router-view />
  </template>
  
  <!-- Main App with Sidebar -->
  <template v-else>
    <Sidebar />
    <main class="main-content">
      <!-- Gateway Status Bar -->
      <GatewayStatusBar :compact="true" />
      
      <router-view v-slot="{ Component }">
        <transition name="fade" mode="out-in">
          <component :is="Component" />
        </transition>
      </router-view>
    </main>
  </template>
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
