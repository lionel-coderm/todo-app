<script setup lang="ts">
import { PRIORITY_LEVELS, getPriorityMeta, type TaskPriority, type CategoryItem } from '@/data/todos';

const props = defineProps<{
  visible: boolean;
  editingTaskId: number | null;
  categories: CategoryItem[];
  title: string;
  categoryId: string;
  priority: TaskPriority;
  description: string;
  saveSuccess: boolean;
}>();

const emit = defineEmits<{
  close: [];
  submit: [];
  'update:title': [value: string];
  'update:categoryId': [value: string];
  'update:priority': [value: TaskPriority];
  'update:description': [value: string];
}>();

function updateTitle(event: Event) {
  emit('update:title', (event.target as HTMLInputElement).value);
}

function updateCategory(event: Event) {
  emit('update:categoryId', (event.target as HTMLSelectElement).value);
}

function updatePriority(event: Event) {
  emit('update:priority', Number((event.target as HTMLSelectElement).value) as TaskPriority);
}

function updateDescription(event: Event) {
  emit('update:description', (event.target as HTMLTextAreaElement).value);
}

const priorityOptions = PRIORITY_LEVELS.map((level) => {
  const meta = getPriorityMeta(level);
  return {
    value: level,
    label: `${meta.label} (${meta.enLabel})`,
  };
});
</script>

<template>
  <div v-if="props.saveSuccess" class="floating-toast toast-success">
    ✅ 任务已保存
  </div>
  <Transition name="fade-up">
    <div v-if="props.visible" class="modal-overlay" @click.self="emit('close')">
      <div class="modal-content">
        <header class="modal-header">
          <h2>{{ props.editingTaskId !== null ? '编辑任务' : '新建任务' }}</h2>
          <button class="close-btn" type="button" @click="emit('close')">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M18 6L6 18M6 6l12 12"></path>
            </svg>
          </button>
        </header>
        <div class="modal-body">
          <div class="form-group">
            <label>任务标题</label>
            <input :value="props.title" type="text" placeholder="准备做什么？" @input="updateTitle" @keyup.enter="emit('submit')" autofocus />
          </div>
          <div class="form-group">
            <label>详细描述 (可选)</label>
            <textarea :value="props.description" placeholder="添加更多细节..." rows="5" @input="updateDescription"></textarea>
          </div>
          <div class="form-row">
            <div class="form-group">
              <label>分类</label>
              <select :value="props.categoryId" @change="updateCategory">
                <option value="" disabled>请选择分类</option>
                <option v-for="cat in props.categories" :key="cat.id" :value="cat.id">
                  {{ cat.icon }} {{ cat.name }}
                </option>
              </select>
            </div>
            <div class="form-group">
              <label>优先级</label>
              <select :value="props.priority" @change="updatePriority">
                <option v-for="option in priorityOptions" :key="option.value" :value="option.value">
                  {{ option.label }}
                </option>
              </select>
            </div>
          </div>
        </div>
        <footer class="modal-footer">
          <button class="btn-ghost" type="button" @click="emit('close')">取消</button>
          <button class="btn-primary" type="button" @click="emit('submit')" :disabled="!props.title.trim() || !props.categoryId">
            {{ props.editingTaskId !== null ? '保存任务' : '添加任务' }}
          </button>
        </footer>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.floating-toast {
  position: fixed;
  top: 28px;
  right: 28px;
  z-index: 1100;
  padding: 12px 16px;
  border-radius: 12px;
  box-shadow: 0 16px 36px rgba(0, 0, 0, 0.16);
  font-size: 0.9rem;
  font-weight: 600;
  backdrop-filter: blur(10px);
}

.toast-success {
  background: rgba(48, 209, 88, 0.16);
  border: 1px solid rgba(48, 209, 88, 0.28);
  color: var(--c-success);
}
</style>
