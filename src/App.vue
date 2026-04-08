<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue';
import { storeToRefs } from 'pinia';
import { type TodoItem as TodoItemModel, TaskPriority } from '@/data/todos';
import type { SearchFilter } from '@/services/storageService';
import { getDefaultDataDir } from '@/services/storageService';
import { useTodoStore } from '@/stores/todo';
import Sidebar from '@/components/Sidebar.vue';
import HeaderBar from '@/components/HeaderBar.vue';
import TodoItem from '@/components/TodoItem.vue';
import TaskTimeline from '@/components/TaskTimeline.vue';
import TaskModal from '@/components/TaskModal.vue';
import CategoryModal from '@/components/CategoryModal.vue';
import DeleteCategoryConfirm from '@/components/DeleteCategoryConfirm.vue';
import TaskDetailModal from '@/components/TaskDetailModal.vue';
import SettingsModal from '@/components/SettingsModal.vue';

const todoStore = useTodoStore();
const {
  filter,
  selectedCategoryId,
  categories,
  visibleTodos,
  stats,
  searchQuery,
  currentSettings,
  sortOrder,
  todos,
} = storeToRefs(todoStore);
const {
  toggleTodo,
  updateSettings,
  updateCategory,
  updateTodo,
  moveToTrash,
  restoreTodo,
  permanentlyDeleteTodo,
} = todoStore;

onMounted(() => {
  todoStore.initialize();
});

const filterTitleMap: Record<SearchFilter, string> = {
  all: '全部任务',
  active: '待处理',
  completed: '已完成',
  trash: '回收站',
};

const currentFilterTitle = computed(() => {
  if (selectedCategoryId.value) {
    return categories.value.find((c) => c.id === selectedCategoryId.value)?.name || '分类列表';
  }
  return filterTitleMap[filter.value] || '待办列表';
});

const defaultTaskCategoryId = computed(() => categories.value[0]?.id || '');

const isTaskModalOpen = ref(false);
const editingTaskId = ref<number | null>(null);
const taskSaveSuccess = ref(false);
let taskSaveSuccessTimer: ReturnType<typeof setTimeout> | null = null;
const taskForm = ref({
  title: '',
  categoryId: '',
  priority: TaskPriority.Low as TaskPriority,
  description: '',
});

onUnmounted(() => {
  if (taskSaveSuccessTimer) {
    clearTimeout(taskSaveSuccessTimer);
    taskSaveSuccessTimer = null;
  }
});

watch(
  defaultTaskCategoryId,
  (categoryId) => {
    if (!editingTaskId.value && !taskForm.value.categoryId && categoryId) {
      taskForm.value.categoryId = categoryId;
    }
  },
  { immediate: true },
);

function resetTaskForm() {
  editingTaskId.value = null;
  taskForm.value = {
    title: '',
    categoryId: defaultTaskCategoryId.value,
    priority: TaskPriority.Low,
    description: '',
  };
}

function openTaskModal() {
  taskSaveSuccess.value = false;
  resetTaskForm();
  isTaskModalOpen.value = true;
}

function closeTaskModal() {
  isTaskModalOpen.value = false;
  resetTaskForm();
}

function startEditTask(todo: TodoItemModel) {
  taskSaveSuccess.value = false;
  editingTaskId.value = todo.id;
  taskForm.value = {
    title: todo.title,
    categoryId: todo.categoryId,
    priority: todo.priority,
    description: todo.description || '',
  };
  closeTaskDetail();
  isTaskModalOpen.value = true;
}

function submitTask() {
  if (!taskForm.value.title.trim()) return;
  if (!taskForm.value.categoryId) return;

  const isEditingTask = editingTaskId.value !== null;

  if (isEditingTask) {
    const taskId = editingTaskId.value;
    if (taskId === null) return;

    updateTodo(
      taskId,
      taskForm.value.title.trim(),
      taskForm.value.priority,
      taskForm.value.categoryId,
      taskForm.value.description.trim(),
    );

    taskSaveSuccess.value = true;
    if (taskSaveSuccessTimer) clearTimeout(taskSaveSuccessTimer);
    taskSaveSuccessTimer = setTimeout(() => {
      taskSaveSuccess.value = false;
      taskSaveSuccessTimer = null;
    }, 1800);
  } else {
    todoStore.addTodo(
      taskForm.value.title.trim(),
      taskForm.value.priority,
      taskForm.value.categoryId,
      taskForm.value.description.trim(),
    );
  }

  closeTaskModal();
}

const isCategoryModalOpen = ref(false);
const presetColors = ['#0A84FF', '#30D158', '#FF9F0A', '#FF453A', '#98989D', '#AF52DE'];
const presetCategoryIcons = ['📁', '💼', '💻', '📚', '🏠', '🛒', '🏃', '💡', '🎯', '✈️', '🎵', '🍽️'];
const editingCategoryId = ref<string | null>(null);
const categoryForm = ref({ name: '', color: '#0A84FF', icon: '📁' });

function resetCategoryForm() {
  editingCategoryId.value = null;
  categoryForm.value = { name: '', color: '#0A84FF', icon: '📁' };
}

function openCategoryModal() {
  isCategoryModalOpen.value = true;
}

function closeCategoryModal() {
  isCategoryModalOpen.value = false;
  resetCategoryForm();
}

function startEditCategory(categoryId: string) {
  const target = categories.value.find((category) => category.id === categoryId);
  if (!target) return;
  editingCategoryId.value = categoryId;
  categoryForm.value = {
    name: target.name,
    color: target.color,
    icon: target.icon,
  };
}

function cancelEditCategory() {
  resetCategoryForm();
}

function submitCategory() {
  if (!categoryForm.value.name.trim() || !categoryForm.value.icon.trim()) return;

  if (editingCategoryId.value) {
    updateCategory(
      editingCategoryId.value,
      categoryForm.value.name.trim(),
      categoryForm.value.color,
      categoryForm.value.icon.trim(),
    );
  } else {
    todoStore.addCategory(
      categoryForm.value.name.trim(),
      categoryForm.value.color,
      categoryForm.value.icon.trim(),
    );
  }

  resetCategoryForm();
}

const isDeleteCategoryConfirmOpen = ref(false);
const pendingDeleteCategoryId = ref<string | null>(null);
const pendingDeleteCategoryName = ref('');
const pendingDeleteCategoryTaskCount = ref(0);

function closeDeleteCategoryConfirm() {
  isDeleteCategoryConfirmOpen.value = false;
  pendingDeleteCategoryId.value = null;
  pendingDeleteCategoryName.value = '';
  pendingDeleteCategoryTaskCount.value = 0;
}

function handleDeleteCategory(categoryId: string) {
  const target = categories.value.find((category) => category.id === categoryId);
  if (!target) return;

  pendingDeleteCategoryId.value = categoryId;
  pendingDeleteCategoryName.value = target.name;
  pendingDeleteCategoryTaskCount.value = todos.value.filter((todo) => todo.categoryId === categoryId).length;
  isDeleteCategoryConfirmOpen.value = true;
}

function confirmDeleteCategory() {
  const categoryId = pendingDeleteCategoryId.value;
  if (!categoryId) return;

  todoStore.deleteCategory(categoryId);

  if (editingCategoryId.value === categoryId) {
    resetCategoryForm();
  }

  closeDeleteCategoryConfirm();
}

const isSettingsModalOpen = ref(false);
const isSavingSettings = ref(false);
const settingsSaveError = ref<string | null>(null);
const settingsSaveSuccess = ref(false);
const defaultDataDirPlaceholder = ref('加载中...');

const settingsForm = ref({
  theme: 'light',
  dataLocation: '' as string,
  dataFormat: 'json' as 'json' | 'sqlite',
  aiModel: '',
  aiBaseUrl: '',
  aiApiKey: '',
});

async function openSettingsModal() {
  settingsForm.value.dataFormat = currentSettings.value.storageType;
  settingsForm.value.dataLocation = currentSettings.value.dataDir ?? '';
  settingsSaveError.value = null;
  settingsSaveSuccess.value = false;
  try {
    defaultDataDirPlaceholder.value = await getDefaultDataDir();
  } catch {
    defaultDataDirPlaceholder.value = '~/Library/Application Support/com.todo.studio';
  }
  isSettingsModalOpen.value = true;
}

function closeSettingsModal() {
  isSettingsModalOpen.value = false;
}

async function handleSaveSettings() {
  if (isSavingSettings.value) return;
  settingsSaveError.value = null;
  settingsSaveSuccess.value = false;
  isSavingSettings.value = true;

  try {
    await updateSettings(settingsForm.value.dataFormat, settingsForm.value.dataLocation);
    settingsSaveSuccess.value = true;
    setTimeout(() => {
      settingsSaveSuccess.value = false;
    }, 2000);
  } catch (err) {
    settingsSaveError.value = String(err);
  } finally {
    isSavingSettings.value = false;
  }
}

const viewTaskData = ref<TodoItemModel | null>(null);

function openTaskDetail(todo: TodoItemModel) {
  viewTaskData.value = todo;
}

function closeTaskDetail() {
  viewTaskData.value = null;
}

function handleToggleFromDetail(id: number) {
  toggleTodo(id);
  closeTaskDetail();
}

function handleSelectFilter(value: SearchFilter) {
  filter.value = value;
  selectedCategoryId.value = null;
}

function handleSelectCategory(id: string | null) {
  selectedCategoryId.value = id;
  if (id) filter.value = 'all';
}
</script>

<template>
  <div class="layout-app">
    <Sidebar
      :filter="filter"
      :selected-category-id="selectedCategoryId"
      :categories="categories"
      :stats="stats"
      @open-new-task="openTaskModal"
      @open-category-modal="openCategoryModal"
      @select-filter="handleSelectFilter"
      @select-category="handleSelectCategory"
    />

    <main class="layout-main">
      <HeaderBar
        :title="currentFilterTitle"
        :total="visibleTodos.length"
        :sort-order="sortOrder"
        :search-query="searchQuery"
        @update:sort-order="sortOrder = $event"
        @update:search-query="searchQuery = $event"
        @open-settings="openSettingsModal"
      />

      <div class="layout-content">
        <div class="content-wrapper">
          <div class="main-list-column">
            <TransitionGroup name="fade-up" tag="ul" class="todo-list">
              <TodoItem
                v-for="todo in visibleTodos"
                :key="todo.id"
                :todo="todo"
                :is-trash-view="filter === 'trash'"
                @toggle="toggleTodo"
                @edit="startEditTask"
                @view="openTaskDetail"
                @delete="moveToTrash"
                @restore="restoreTodo"
                @permanent-delete="permanentlyDeleteTodo"
              />

              <li v-if="visibleTodos.length === 0" key="empty" class="empty-state">
                <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="var(--c-border)" stroke-width="1.5">
                  <rect x="3" y="4" width="18" height="18" rx="2" ry="2"></rect>
                  <line x1="16" y1="2" x2="16" y2="6"></line>
                  <line x1="8" y1="2" x2="8" y2="6"></line>
                  <line x1="3" y1="10" x2="21" y2="10"></line>
                </svg>
                <p>没有找到相关任务内容...</p>
              </li>
            </TransitionGroup>
          </div>

          <aside class="timeline-column">
            <TaskTimeline />
          </aside>
        </div>
      </div>
    </main>

    <Teleport to="body">
      <TaskModal
        :visible="isTaskModalOpen"
        :editing-task-id="editingTaskId"
        :categories="categories"
        :title="taskForm.title"
        :category-id="taskForm.categoryId"
        :priority="taskForm.priority"
        :description="taskForm.description"
        :save-success="taskSaveSuccess"
        @close="closeTaskModal"
        @submit="submitTask"
        @update:title="taskForm.title = $event"
        @update:category-id="taskForm.categoryId = $event"
        @update:priority="taskForm.priority = $event"
        @update:description="taskForm.description = $event"
      />

      <CategoryModal
        :visible="isCategoryModalOpen"
        :categories="categories"
        :editing-category-id="editingCategoryId"
        :name="categoryForm.name"
        :color="categoryForm.color"
        :icon="categoryForm.icon"
        :preset-colors="presetColors"
        :preset-icons="presetCategoryIcons"
        @close="closeCategoryModal"
        @submit="submitCategory"
        @cancel-edit="cancelEditCategory"
        @start-edit="startEditCategory"
        @request-delete="handleDeleteCategory"
        @update:name="categoryForm.name = $event"
        @update:color="categoryForm.color = $event"
        @update:icon="categoryForm.icon = $event"
      />

      <DeleteCategoryConfirm
        :visible="isDeleteCategoryConfirmOpen"
        :category-name="pendingDeleteCategoryName"
        :task-count="pendingDeleteCategoryTaskCount"
        @close="closeDeleteCategoryConfirm"
        @confirm="confirmDeleteCategory"
      />

      <TaskDetailModal
        :visible="!!viewTaskData"
        :task="viewTaskData"
        :categories="categories"
        @close="closeTaskDetail"
        @edit="startEditTask"
        @toggle="handleToggleFromDetail"
      />

      <SettingsModal
        :visible="isSettingsModalOpen"
        :theme="settingsForm.theme"
        :data-location="settingsForm.dataLocation"
        :data-format="settingsForm.dataFormat"
        :ai-model="settingsForm.aiModel"
        :ai-base-url="settingsForm.aiBaseUrl"
        :ai-api-key="settingsForm.aiApiKey"
        :default-data-dir-placeholder="defaultDataDirPlaceholder"
        :is-saving="isSavingSettings"
        :error="settingsSaveError"
        :success="settingsSaveSuccess"
        @close="closeSettingsModal"
        @save="handleSaveSettings"
        @update:theme="settingsForm.theme = $event"
        @update:data-location="settingsForm.dataLocation = $event"
        @update:data-format="settingsForm.dataFormat = $event"
        @update:ai-model="settingsForm.aiModel = $event"
        @update:ai-base-url="settingsForm.aiBaseUrl = $event"
        @update:ai-api-key="settingsForm.aiApiKey = $event"
      />
    </Teleport>
  </div>
</template>

<style scoped>
.content-wrapper {
  display: flex;
  gap: 40px;
  align-items: flex-start;
  height: 100%;
}

.main-list-column {
  flex: 1;
  min-width: 0;
}

.timeline-column {
  width: 160px;
  flex-shrink: 0;
}

.todo-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-width: 1000px;
  position: relative;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 16px;
  padding: 40px;
  color: var(--c-text-muted);
  border: 1px dashed var(--c-border);
  border-radius: var(--radius-md);
  margin-top: 20px;
}
</style>
