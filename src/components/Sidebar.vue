<script setup lang="ts">
import { useRoute, useRouter } from 'vue-router'
import { computed } from 'vue'
import { useAgentStore } from '@/stores/agents'

const route = useRoute()
const router = useRouter()
const agentStore = useAgentStore()

const navItems = [
  { path: '/', name: 'dashboard', label: '仪表盘', icon: '📊', section: 'main' },
  { path: '/agents', name: 'agents', label: '特工列队', icon: '🤖', section: 'main', badge: true },
  { path: '/overwatch', name: 'overwatch', label: '监控舱', icon: '📺', section: 'main' },
  { path: '/cost', name: 'cost', label: '烧钱计算器', icon: '💰', section: 'main' },
  { path: '/templates', name: 'templates', label: '模板市场', icon: '🎯', section: 'main' },
  { path: '/sandboxes', name: 'sandboxes', label: '沙盒环境', icon: '📦', section: 'system' },
  { path: '/traces', name: 'traces', label: '历史回放', icon: '⏱', section: 'system' },
  { path: '/audit', name: 'audit', label: '合规审计 🔐', icon: '📋', section: 'system', badge: 'EE' },
  { path: '/settings', name: 'settings', label: '系统设置', icon: '⚙', section: 'system' },
]

const isActive = (name: string) => {
  return route.name === name || (name === 'overwatch' && route.name === 'overwatch-agent')
}

const agentCount = computed(() => agentStore.runningAgents.length)
</script>

<template>
  <aside class="sidebar">
    <div class="sidebar-logo">
      <div class="logo-mark">
        <div class="logo-icon">🗜</div>
        <div>
          <h1>ClawStudio</h1>
          <div class="version">Nova v0.1</div>
        </div>
      </div>
    </div>

    <nav class="sidebar-nav">
      <div class="nav-section-label">Main</div>
      <template v-for="item in navItems" :key="item.name">
        <div v-if="item.section === 'system' && navItems[navItems.indexOf(item) - 1]?.section === 'main'"
             class="nav-section-label">System</div>
        <div
          class="nav-item"
          :class="{ active: isActive(item.name) }"
          @click="router.push(item.path)"
        >
          <span class="nav-icon">{{ item.icon }}</span>
          {{ item.label }}
          <span v-if="item.badge && agentCount > 0" class="badge">{{ agentCount }}</span>
        </div>
      </template>
    </nav>

    <div class="sidebar-footer">
      <div class="avatar">🐱</div>
      <div class="user-info">
        <div class="name">Nova Admin</div>
        <div class="role">本地实例</div>
      </div>
    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  width: var(--sidebar-w);
  min-width: var(--sidebar-w);
  height: 100vh;
  background: var(--bg-base);
  border-right: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  position: relative;
}
.sidebar::after {
  content: '';
  position: absolute;
  right: 0; top: 0; bottom: 0;
  width: 1px;
  background: linear-gradient(180deg, var(--cyan-dim), transparent 30%, transparent 70%, var(--cyan-dim));
  opacity: 0.4;
}

.sidebar-logo { padding: 20px 18px 16px; border-bottom: 1px solid var(--border); }
.logo-mark { display: flex; align-items: center; gap: 10px; }
.logo-icon {
  width: 36px; height: 36px;
  background: linear-gradient(135deg, var(--cyan), #0891b2);
  border-radius: 8px;
  display: flex; align-items: center; justify-content: center;
  font-size: 18px;
  box-shadow: 0 0 20px var(--cyan-glow);
}
h1 {
  font-size: 17px; font-weight: 700; letter-spacing: 0.5px;
  background: linear-gradient(135deg, var(--cyan), #67e8f9);
  -webkit-background-clip: text; -webkit-text-fill-color: transparent;
}
.version { font-size: 10px; color: var(--text-dim); font-family: var(--font-mono); }

.sidebar-nav { flex: 1; padding: 12px 10px; display: flex; flex-direction: column; gap: 2px; }
.nav-section-label {
  font-size: 10px; text-transform: uppercase; color: var(--text-dim);
  letter-spacing: 1.5px; padding: 16px 14px 6px; font-weight: 600;
}
.nav-item {
  display: flex; align-items: center; gap: 12px;
  padding: 10px 14px; border-radius: var(--radius-sm);
  cursor: pointer; transition: all 0.2s;
  color: var(--text-secondary); font-size: 13px; font-weight: 500;
  position: relative;
}
.nav-item:hover { background: var(--bg-card); color: var(--text-primary); }
.nav-item.active {
  background: var(--cyan-glow); color: var(--cyan);
  box-shadow: inset 0 0 0 1px rgba(6,214,214,0.15);
}
.nav-item.active::before {
  content: ''; position: absolute; left: 0; top: 50%;
  transform: translateY(-50%); width: 3px; height: 20px;
  background: var(--cyan); border-radius: 0 3px 3px 0;
}
.nav-icon { font-size: 17px; width: 22px; text-align: center; }
.badge {
  margin-left: auto; background: var(--red); color: #fff;
  font-size: 10px; padding: 1px 6px; border-radius: 10px; font-weight: 600;
}

.sidebar-footer {
  padding: 14px; border-top: 1px solid var(--border);
  display: flex; align-items: center; gap: 10px;
}
.avatar {
  width: 32px; height: 32px; border-radius: 50%;
  background: linear-gradient(135deg, #6366f1, #8b5cf6);
  display: flex; align-items: center; justify-content: center;
  font-size: 13px; font-weight: 600;
}
.user-info .name { font-size: 12px; font-weight: 600; }
.user-info .role { font-size: 10px; color: var(--text-dim); }
</style>
