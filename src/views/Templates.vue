<template>
  <div class="templates-page">
    <TemplateBrowser />
  </div>
</template>

<script setup lang="ts">
import TemplateBrowser from '@/components/TemplateBrowser.vue'
</script>

<style scoped>
.templates-page {
  min-height: 100%;
  background: linear-gradient(135deg, #0a0a0f 0%, #1a1a2e 100%);
}
</style>