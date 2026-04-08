<script setup lang="ts">
defineProps<{
  title: string;
  value: string | number;
  hint?: string; // 保留 hint 以防其他地方调用时报错，但侧边栏设计中可以不用
}>();
</script>

<template>
  <div class="stat-card" :title="hint">
    <span class="stat-title">{{ title }}</span>
    <span class="stat-value">{{ value }}</span>
  </div>
</template>

<style scoped>
.stat-card {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  background-color: var(--c-bg-app);
  border-radius: var(--radius-sm);
  border: 1px solid var(--c-border);
  transition: var(--transition-fast);
}

.stat-card:hover {
  border-color: var(--c-border-light);
  transform: translateY(-1px);
  box-shadow: var(--shadow-sm);
}

.stat-title {
  font-size: 0.85rem;
  color: var(--c-text-secondary);
  font-weight: 500;
}

.stat-value {
  font-size: 0.95rem;
  font-weight: 600;
  color: var(--c-text-primary);
}
</style>
