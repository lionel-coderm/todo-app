<script setup lang="ts">
const props = defineProps<{
  title: string;
  total: number;
  sortOrder: 'created' | 'priority';
  searchQuery: string;
}>();

const emit = defineEmits<{
  'update:sortOrder': [value: 'created' | 'priority'];
  'update:searchQuery': [value: string];
  'open-settings': [];
}>();

function updateSearchQuery(event: Event) {
  emit('update:searchQuery', (event.target as HTMLInputElement).value);
}
</script>

<template>
  <header class="layout-header">
    <div class="header-title">
      <h1>{{ props.title }}</h1>
      <p class="header-subtitle">{{ props.total }} 个任务</p>
    </div>
    <div class="header-actions">
      <div class="sort-toggle">
        <button
          class="sort-btn"
          :class="{ 'is-active': props.sortOrder === 'created' }"
          @click="emit('update:sortOrder', 'created')"
          title="按创建时间排序"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <rect x="3" y="4" width="18" height="18" rx="2" ry="2"></rect>
            <line x1="16" y1="2" x2="16" y2="6"></line>
            <line x1="8" y1="2" x2="8" y2="6"></line>
            <line x1="3" y1="10" x2="21" y2="10"></line>
          </svg>
          时间
        </button>
        <button
          class="sort-btn"
          :class="{ 'is-active': props.sortOrder === 'priority' }"
          @click="emit('update:sortOrder', 'priority')"
          title="按优先级排序"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M3 6h18M7 12h10M11 18h2"></path>
          </svg>
          优先级
        </button>
      </div>
      <div class="search-box">
        <svg class="search-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="11" cy="11" r="8"></circle>
          <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
        </svg>
        <input :value="props.searchQuery" @input="updateSearchQuery" type="text" placeholder="搜索任务..." />
      </div>
      <button
        class="icon-btn settings-btn"
        @click="emit('open-settings')"
        title="系统设置"
      >
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="3"></circle>
          <path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-2 2 2 2 0 01-2-2v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83 0 2 2 0 010-2.83l.06-.06a1.65 1.65 0 00.33-1.82 1.65 1.65 0 00-1.51-1H3a2 2 0 01-2-2 2 2 0 012-2h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 010-2.83 2 2 0 012.83 0l.06.06a1.65 1.65 0 001.82.33H9a1.65 1.65 0 001-1.51V3a2 2 0 012-2 2 2 0 012 2v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 0 2 2 0 010 2.83l-.06.06a1.65 1.65 0 00-.33 1.82V9a1.65 1.65 0 001.51 1H21a2 2 0 012 2 2 2 0 01-2 2h-.09a1.65 1.65 0 00-1.51 1z"></path>
        </svg>
      </button>
    </div>
  </header>
</template>

<style scoped>
.layout-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.header-actions {
  display: flex;
  align-items: center;
}

.header-title h1 {
  font-size: 1.8rem;
  font-weight: 700;
  margin-bottom: 4px;
}

.header-subtitle {
  color: var(--c-text-secondary);
  font-size: 0.95rem;
}

.search-box {
  position: relative;
  display: flex;
  align-items: center;
}

.search-icon {
  position: absolute;
  left: 12px;
  color: var(--c-text-muted);
}

.search-box input {
  padding: 8px 16px 8px 36px;
  width: 240px;
  height: 38px;
  border-radius: 99px;
  border: 1px solid var(--c-border);
  background-color: var(--c-bg-surface);
  color: var(--c-text-primary);
  font-size: 0.9rem;
  transition: all 0.2s;
  outline: none;
}

.search-box input:focus {
  border-color: var(--c-accent);
  box-shadow: 0 0 0 3px rgba(10, 132, 255, 0.15);
  width: 280px;
}

.sort-toggle {
  display: flex;
  align-items: center;
  gap: 2px;
  background-color: var(--c-bg-app);
  border: 1px solid var(--c-border);
  border-radius: var(--radius-sm);
  padding: 3px;
  margin-right: 12px;
}

.sort-btn {
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 5px 10px;
  border-radius: calc(var(--radius-sm) - 2px);
  font-size: 0.8rem;
  font-weight: 500;
  color: var(--c-text-secondary);
  background: transparent;
  border: none;
  cursor: pointer;
  transition: var(--transition-fast);
  white-space: nowrap;
}

.sort-btn:hover {
  color: var(--c-text-primary);
  background-color: var(--c-bg-sidebar-hover);
}

.sort-btn.is-active {
  background-color: var(--c-accent);
  color: #fff;
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

.settings-btn {
  margin-left: 16px;
  background-color: var(--c-bg-surface);
  width: 38px;
  height: 38px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--c-border);
  display: flex;
  align-items: center;
  justify-content: center;
}
</style>
