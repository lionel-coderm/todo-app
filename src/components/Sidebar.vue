<script setup lang="ts">
import StatCard from '@/components/StatCard.vue';
import type { CategoryItem } from '@/data/todos';
import type { SearchFilter } from '@/services/storageService';

interface Stats {
  total: number;
  completed: number;
  pending: number;
  trash: number;
}

const props = defineProps<{
  filter: SearchFilter;
  selectedCategoryId: string | null;
  categories: CategoryItem[];
  stats: Stats;
}>();

const emit = defineEmits<{
  'open-new-task': [];
  'open-category-modal': [];
  'select-filter': [value: SearchFilter];
  'select-category': [id: string | null];
}>();

const filterOptions: Array<{ label: string; value: SearchFilter; icon: string }> = [
  { label: '全部任务', value: 'all', icon: 'M4 6h16M4 12h16M4 18h7' },
  { label: '待处理', value: 'active', icon: 'M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z' },
  { label: '已完成', value: 'completed', icon: 'M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z' },
  { label: '回收站', value: 'trash', icon: 'M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16' },
];

function handleFilterClick(value: SearchFilter) {
  emit('select-filter', value);
  emit('select-category', null);
}

function handleCategoryClick(id: string) {
  emit('select-category', id);
}
</script>

<template>
  <aside class="layout-sidebar">
    <div class="sidebar-header">
      <div class="logo">
        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="var(--c-accent)" stroke-width="2">
          <path d="M5 12l5 5L20 7" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
        <h2>Todo Studio</h2>
      </div>
    </div>

    <div class="sidebar-action">
      <button class="btn-primary new-task-btn" @click="emit('open-new-task')">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <path d="M12 5v14M5 12h14"></path>
        </svg>
        新建任务
      </button>
    </div>

    <nav class="sidebar-nav">
      <p class="nav-title">过滤器</p>
      <button
        v-for="option in filterOptions"
        :key="option.value"
        class="nav-item"
        :class="{ 'is-active': filter === option.value && !selectedCategoryId }"
        @click="handleFilterClick(option.value)"
      >
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path :d="option.icon"></path>
        </svg>
        {{ option.label }}
        <span class="nav-count" v-if="option.value === 'all'">{{ props.stats.total }}</span>
        <span class="nav-count" v-else-if="option.value === 'active'">{{ props.stats.pending }}</span>
        <span class="nav-count" v-else-if="option.value === 'completed'">{{ props.stats.completed }}</span>
        <span class="nav-count" v-else>{{ props.stats.trash }}</span>
      </button>
      <div class="nav-divider"></div>
      <div class="nav-section-header">
        <p class="nav-title">自定义分类</p>
        <button class="icon-btn" @click="emit('open-category-modal')" title="管理分类">+</button>
      </div>
      <button
        v-for="cat in categories"
        :key="cat.id"
        class="nav-item"
        :class="{ 'is-active': selectedCategoryId === cat.id }"
        @click="handleCategoryClick(cat.id)"
      >
        <span class="cat-icon" :style="{ color: cat.color }">{{ cat.icon }}</span>
        {{ cat.name }}
      </button>
    </nav>

    <div class="sidebar-bottom">
      <p class="nav-title">数据统计</p>
      <div class="stats-container">
        <StatCard title="总任务" :value="props.stats.total" />
        <StatCard title="待办" :value="props.stats.pending" />
        <StatCard title="已完成" :value="props.stats.completed" />
      </div>
    </div>
  </aside>
</template>

<style scoped>
.sidebar-header {
  margin-bottom: 24px;
}

.logo {
  display: flex;
  align-items: center;
  gap: 12px;
}

.logo h2 {
  font-size: 1.1rem;
  font-weight: 600;
  color: var(--c-text-primary);
}

.sidebar-action {
  margin-bottom: 32px;
}

.new-task-btn {
  width: 100%;
}

.nav-title {
  font-size: 0.75rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--c-text-muted);
  margin-bottom: 12px;
  font-weight: 600;
  padding-left: 8px;
}

.sidebar-nav {
  display: flex;
  flex-direction: column;
  gap: 4px;
  flex: 1;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 12px;
  border-radius: var(--radius-sm);
  color: var(--c-text-secondary);
  transition: var(--transition-fast);
  font-size: 0.95rem;
  font-weight: 500;
}

.nav-item:hover {
  background-color: var(--c-bg-sidebar-hover);
  color: var(--c-text-primary);
}

.nav-item.is-active {
  background-color: var(--c-accent-bg);
  color: var(--c-accent);
}

.nav-item svg {
  opacity: 0.8;
}

.nav-count {
  margin-left: auto;
  font-size: 0.8rem;
  background-color: var(--c-bg-app);
  padding: 2px 8px;
  border-radius: 999px;
  color: var(--c-text-muted);
  border: 1px solid var(--c-border);
}

.nav-item.is-active .nav-count {
  color: var(--c-accent);
  border-color: rgba(10, 132, 255, 0.2);
  background-color: var(--c-bg-surface);
}

.sidebar-bottom {
  margin-top: auto;
  padding-top: 24px;
}

.stats-container {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.nav-divider {
  height: 1px;
  background-color: var(--c-border-light);
  margin: 8px 0;
}

.nav-section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-right: 8px;
}

.icon-btn {
  background: none;
  border: none;
  color: var(--c-text-muted);
  font-size: 1.2rem;
  cursor: pointer;
  transition: var(--transition-fast);
}

.icon-btn:hover {
  color: var(--c-text-primary);
}

.cat-icon {
  font-size: 1rem;
  margin-right: -4px;
}
</style>
