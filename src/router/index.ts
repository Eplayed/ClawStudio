import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/setup',
      name: 'setup',
      component: () => import('@/views/SetupWizard.vue'),
      meta: { title: '安装向导', fullscreen: true },
    },
    {
      path: '/',
      name: 'dashboard',
      component: () => import('@/views/Dashboard.vue'),
      meta: { title: '仪表盘', icon: '📊' },
    },
    {
      path: '/agents',
      name: 'agents',
      component: () => import('@/views/Agents.vue'),
      meta: { title: '特工列队', icon: '🤖' },
    },
    {
      path: '/overwatch',
      name: 'overwatch',
      component: () => import('@/views/Overwatch.vue'),
      meta: { title: '监控舱', icon: '📺' },
    },
    {
      path: '/overwatch/:agentId',
      name: 'overwatch-agent',
      component: () => import('@/views/Overwatch.vue'),
      meta: { title: '监控舱', icon: '📺' },
    },
    {
      path: '/cost',
      name: 'cost',
      component: () => import('@/views/CostMonitor.vue'),
      meta: { title: '烧钱计算器', icon: '💰' },
    },
    {
      path: '/sandboxes',
      name: 'sandboxes',
      component: () => import('@/views/Sandboxes.vue'),
      meta: { title: '沙盒环境', icon: '📦' },
    },
    {
      path: '/traces',
      name: 'traces',
      component: () => import('@/views/Traces.vue'),
      meta: { title: '历史回放', icon: '⏱' },
    },
    {
      path: '/audit',
      name: 'audit',
      component: () => import('@/views/AuditTraces.vue'),
      meta: { title: '合规审计', icon: '📋' },
    },
    {
      path: '/templates',
      name: 'templates',
      component: () => import('@/views/Templates.vue'),
      meta: { title: '模板市场', icon: '🎯' },
    },
    {
      path: '/settings',
      name: 'settings',
      component: () => import('@/views/Settings.vue'),
      meta: { title: '系统设置', icon: '⚙' },
    },
  ],
})

export default router
