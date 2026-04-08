<script setup lang="ts">
import { computed } from 'vue';
import { getPriorityMeta, type CategoryItem, type TodoItem } from '@/data/todos';
import { formatDateMonthDay, formatDateYmdHm } from '@/utils/dateFormat';

const props = defineProps<{
  visible: boolean;
  task: TodoItem | null;
  categories: CategoryItem[];
}>();

const emit = defineEmits<{
  close: [];
  edit: [todo: TodoItem];
  toggle: [id: number];
}>();

const taskCategory = computed(() => {
  if (!props.task) return null;
  return props.categories.find((item) => item.id === props.task?.categoryId) || null;
});
const taskPriorityMeta = computed(() => getPriorityMeta(props.task?.priority));
const taskPriorityStyle = computed(() => ({
  backgroundColor: taskPriorityMeta.value.bgColor,
  color: taskPriorityMeta.value.color,
  padding: '4px 10px',
  borderRadius: '4px',
  fontSize: '0.8rem',
  fontWeight: 600,
}));
</script>

<template>
  <Transition name="fade-up">
    <div v-if="props.visible && props.task" class="modal-overlay" @click.self="emit('close')">
      <div class="modal-content detail-modal-content">
        <header class="modal-header">
          <h2>任务详情</h2>
          <button class="close-btn" type="button" @click="emit('close')">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M18 6L6 18M6 6l12 12"></path>
            </svg>
          </button>
        </header>
        <div class="modal-body detail-body">
          <div class="detail-header-block">
            <h1 class="detail-title">{{ props.task.title }}</h1>
            <div class="meta detail-meta">
              <span
                class="pill"
                v-if="taskCategory"
                :style="{
                  backgroundColor: taskCategory.color + '20',
                  color: taskCategory.color,
                  padding: '4px 10px',
                  borderRadius: '4px',
                  fontSize: '0.8rem',
                  fontWeight: 600,
                }"
              >
                {{ taskCategory.icon }} {{ taskCategory.name }}
              </span>
              <span
                class="pill"
                v-else
                style="background-color: var(--c-bg-sidebar-hover); color: var(--c-text-secondary); padding: 4px 10px; border-radius: 4px"
              >
                未分类
              </span>

              <span class="pill" :style="taskPriorityStyle">
                {{ taskPriorityMeta.label }}
              </span>
              <span class="detail-short-date">{{ formatDateMonthDay(props.task.createdAt) }}</span>
            </div>
          </div>

          <div class="detail-grid">
            <div class="detail-card">
              <div class="detail-card-title">创建日期</div>
              <div class="detail-card-value">{{ formatDateYmdHm(props.task.createdAt) }}</div>
            </div>
            <div class="detail-card">
              <div class="detail-card-title">完成日期</div>
              <div class="detail-card-value" :class="{ 'is-completed': props.task.completedAt }">
                {{ formatDateYmdHm(props.task.completedAt) }}
              </div>
            </div>
          </div>

          <div class="detail-desc-block">
            <h3 class="detail-desc-title">详细描述</h3>
            <p v-if="props.task.description" class="detail-desc-text">{{ props.task.description }}</p>
            <p v-else class="detail-desc-empty">暂无描述信息...</p>
          </div>
        </div>
        <footer class="modal-footer">
          <div class="task-detail-actions">
            <button class="btn-ghost" type="button" @click="emit('edit', props.task)">编辑任务</button>
            <button class="btn-primary" type="button" v-if="!props.task.completed" @click="emit('toggle', props.task.id)">
              标记为已完成
            </button>
            <button class="btn-ghost" type="button" v-else @click="emit('toggle', props.task.id)">重新激活任务</button>
          </div>
        </footer>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.detail-modal-content {
  max-width: 520px;
}

.detail-body {
  gap: 20px;
}

.detail-title {
  font-size: 1.4rem;
  color: var(--c-text-primary);
  margin-bottom: 12px;
  font-weight: 600;
  line-height: 1.3;
}

.detail-meta {
  display: flex;
  gap: 8px;
  align-items: center;
}

.detail-short-date {
  margin-left: auto;
  font-size: 0.85rem;
  color: var(--c-text-muted);
}

.detail-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
}

.detail-card {
  background-color: var(--c-bg-app);
  border: 1px solid var(--c-border-light);
  border-radius: 8px;
  padding: 14px 16px;
}

.detail-card-title {
  font-size: 0.78rem;
  color: var(--c-text-secondary);
  text-transform: uppercase;
  font-weight: 600;
  margin-bottom: 8px;
}

.detail-card-value {
  font-size: 0.95rem;
  color: var(--c-text-primary);
  font-weight: 500;
}

.detail-card-value.is-completed {
  color: var(--c-success);
}

.detail-desc-block {
  background-color: var(--c-bg-app);
  border: 1px solid var(--c-border-light);
  border-radius: 8px;
  padding: 16px;
  min-height: 240px;
  max-height: 400px;
  overflow-y: auto;
}

.detail-desc-title {
  font-size: 0.85rem;
  margin-bottom: 12px;
  color: var(--c-text-secondary);
  font-weight: 600;
  text-transform: uppercase;
}

.detail-desc-text {
  white-space: pre-wrap;
  font-size: 0.95rem;
  color: var(--c-text-primary);
  line-height: 1.6;
}

.detail-desc-empty {
  color: var(--c-text-muted);
  font-size: 0.9rem;
  font-style: italic;
}

.task-detail-actions {
  display: flex;
  gap: 12px;
  width: 100%;
}

.task-detail-actions .btn-primary,
.task-detail-actions .btn-ghost {
  flex: 1;
}
</style>
