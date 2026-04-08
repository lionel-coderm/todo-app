<script setup lang="ts">
import type { TodoItem } from '@/data/todos';
import { useTodoStore } from '@/stores/todo';
import { computed } from 'vue';

const props = defineProps<{
  todo: TodoItem;
}>();

const emit = defineEmits<{
  toggle: [id: number];
  edit: [todo: TodoItem];
  view: [todo: TodoItem];
}>();

const todoStore = useTodoStore();
const category = computed(() => todoStore.categories.find(c => c.id === props.todo.categoryId));
const createdAtLabel = computed(() => formatDisplayDate(props.todo.createdAt));
const completedAtLabel = computed(() => formatDisplayDate(props.todo.completedAt));

function formatDisplayDate(dateString?: string) {
  if (!dateString) return '—';
  const date = new Date(dateString);
  if (Number.isNaN(date.getTime())) return '—';
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')}`;
}
</script>

<template>
  <li class="todo-item" :class="{ 'is-completed': todo.completed }" @dblclick="emit('view', todo)">
    <button class="checkbox" type="button" @click.stop="emit('toggle', todo.id)">
      <svg class="check-icon" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
        <path d="M5 13L9 17L19 7" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"/>
      </svg>
    </button>

    <div class="content">
      <div class="title-row">
        <h3 class="title">{{ todo.title }}</h3>
        <button class="edit-btn" type="button" @click.stop="emit('edit', todo)">
          编辑
        </button>
      </div>
      <p v-if="todo.description" class="description">{{ todo.description }}</p>
      <div class="meta">
        <span class="pill pill-category" v-if="category" :style="{ backgroundColor: category.color + '20', color: category.color }">
          {{ category.icon }} {{ category.name }}
        </span>
        <span class="pill pill-category" v-else>未分类</span>
        <span class="pill pill-priority" :data-priority="todo.priority || 'low'">
          {{ todo.priority === 'high' ? '高' : todo.priority === 'medium' ? '中' : '低' }}
        </span>
        <span class="date">创建：{{ createdAtLabel }}</span>
        <span v-if="todo.completedAt" class="date date-success">完成：{{ completedAtLabel }}</span>
        <span class="date">{{ todo.dueLabel }}</span>
      </div>
    </div>
  </li>
</template>

<style scoped>
.todo-item {
  display: flex;
  align-items: flex-start;
  gap: 16px;
  padding: 16px;
  background-color: var(--c-bg-surface);
  border: 1px solid var(--c-border);
  border-radius: var(--radius-md);
  transition: var(--transition-normal);
  box-shadow: var(--shadow-sm);
  cursor: pointer;
}

.todo-item:hover {
  transform: translateY(-2px);
  box-shadow: var(--shadow-md);
  border-color: var(--c-border-light);
}

.checkbox {
  width: 24px;
  height: 24px;
  border-radius: 6px;
  border: 2px solid var(--c-text-muted);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  margin-top: 2px;
  transition: var(--transition-fast);
  color: var(--c-bg-surface); /* icon color */
}

.check-icon {
  width: 14px;
  height: 14px;
  stroke-dasharray: 24;
  stroke-dashoffset: 24;
  transition: stroke-dashoffset 0.3s cubic-bezier(0.65, 0, 0.45, 1);
}

.checkbox:hover {
  border-color: var(--c-accent);
}

.todo-item.is-completed {
  opacity: 0.6;
  background-color: var(--c-bg-app);
}

.todo-item.is-completed .checkbox {
  background-color: var(--c-accent);
  border-color: var(--c-accent);
}

.todo-item.is-completed .check-icon {
  stroke-dashoffset: 0;
}

.todo-item.is-completed .title {
  color: var(--c-text-muted);
  text-decoration: line-through;
}

.todo-item.is-completed:hover {
  transform: none;
  box-shadow: var(--shadow-sm);
}

.content {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.title-row {
  display: flex;
  align-items: flex-start;
  gap: 12px;
}

.title {
  font-size: 1rem;
  font-weight: 500;
  color: var(--c-text-primary);
  margin: 0;
  transition: color 0.3s ease;
  flex: 1;
}

.edit-btn {
  border: none;
  background: rgba(10, 132, 255, 0.1);
  color: var(--c-accent);
  border-radius: 999px;
  padding: 4px 10px;
  font-size: 0.75rem;
  font-weight: 600;
  cursor: pointer;
  transition: var(--transition-fast);
}
.edit-btn:hover {
  background: rgba(10, 132, 255, 0.16);
}

.description {
  font-size: 0.85rem;
  color: var(--c-text-secondary);
  margin: 0;
  line-height: 1.4;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.meta {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 8px;
}

.pill {
  font-size: 0.75rem;
  padding: 2px 8px;
  border-radius: var(--radius-sm);
  font-weight: 600;
}

.pill-category {
  background-color: var(--c-bg-sidebar-hover);
  color: var(--c-text-secondary);
}

.pill-priority[data-priority="high"] {
  background-color: rgba(255, 59, 48, 0.1);
  color: var(--c-danger);
}
.pill-priority[data-priority="medium"] {
  background-color: rgba(255, 149, 0, 0.1);
  color: var(--c-warning);
}
.pill-priority[data-priority="low"] {
  background-color: rgba(52, 199, 89, 0.1);
  color: var(--c-success);
}

.date {
  font-size: 0.8rem;
  color: var(--c-text-muted);
}

.date-success {
  color: var(--c-success);
}
</style>
