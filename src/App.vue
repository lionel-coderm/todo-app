<script setup lang="ts">
import { computed, defineAsyncComponent, nextTick, onMounted } from 'vue';
import { storeToRefs } from 'pinia';
import type { SearchFilter } from '@/services/storageService';
import { showMainWindow } from '@/services/storageService';
import { useTodoStore } from '@/stores/todo';
import Sidebar from '@/components/Sidebar.vue';
import HeaderBar from '@/components/HeaderBar.vue';
import TodoItem from '@/components/TodoItem.vue';
import TaskTimeline from '@/components/TaskTimeline.vue';
import { useTaskModal } from '@/composables/useTaskModal';
import { useCategoryModal } from '@/composables/useCategoryModal';
import { useSettingsModal } from '@/composables/useSettingsModal';
import { useTaskDetail } from '@/composables/useTaskDetail';

async function loadModalComponent<T>(loader: () => Promise<T>) {
  await import('@/modals.css');
  return loader();
}

const TaskModal = defineAsyncComponent(() => loadModalComponent(() => import('@/components/TaskModal.vue')));
const CategoryModal = defineAsyncComponent(() => loadModalComponent(() => import('@/components/CategoryModal.vue')));
const DeleteCategoryConfirm = defineAsyncComponent(
  () => loadModalComponent(() => import('@/components/DeleteCategoryConfirm.vue')),
);
const TaskDetailModal = defineAsyncComponent(
  () => loadModalComponent(() => import('@/components/TaskDetailModal.vue')),
);
const SettingsModal = defineAsyncComponent(
  () => loadModalComponent(() => import('@/components/SettingsModal.vue')),
);

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
  storageError,
} = storeToRefs(todoStore);
const {
  toggleTodo,
  moveToTrash,
  restoreTodo,
  permanentlyDeleteTodo,
  clearStorageError,
} = todoStore;

onMounted(() => {
  void todoStore.initialize();
  void nextTick()
    .then(() => showMainWindow())
    .catch(() => undefined);
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

const {
  viewTaskData,
  openTaskDetail,
  closeTaskDetail,
  handleToggleFromDetail,
} = useTaskDetail({
  toggleTodo,
});

const {
  isTaskModalOpen,
  editingTaskId,
  taskSaveSuccess,
  taskForm,
  openTaskModal,
  closeTaskModal,
  startEditTask,
  submitTask,
} = useTaskModal({
  defaultTaskCategoryId,
  addTodo: todoStore.addTodo,
  updateTodo: todoStore.updateTodo,
  onBeforeEditTask: closeTaskDetail,
});

const {
  isCategoryModalOpen,
  editingCategoryId,
  categoryForm,
  isDeleteCategoryConfirmOpen,
  pendingDeleteCategoryName,
  pendingDeleteCategoryTaskCount,
  presetColors,
  presetCategoryIcons,
  openCategoryModal,
  closeCategoryModal,
  startEditCategory,
  cancelEditCategory,
  submitCategory,
  handleDeleteCategory,
  closeDeleteCategoryConfirm,
  confirmDeleteCategory,
} = useCategoryModal({
  categories,
  todos,
  addCategory: todoStore.addCategory,
  updateCategory: todoStore.updateCategory,
  deleteCategory: todoStore.deleteCategory,
});

const {
  isSettingsModalOpen,
  isSavingSettings,
  settingsSaveError,
  settingsSaveSuccess,
  defaultDataDirPlaceholder,
  isGeneratingReport,
  reportError,
  reportContent,
  reportPeriod,
  settingsForm,
  openSettingsModal,
  closeSettingsModal,
  handleSaveSettings,
  handleGenerateReport,
} = useSettingsModal({
  currentSettings,
  updateSettings: todoStore.updateSettings,
});

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

      <div v-if="storageError" class="storage-error-bar">
        <span class="storage-error-text">存储操作失败: {{ storageError }}</span>
        <button class="storage-error-dismiss" @click="clearStorageError">&times;</button>
      </div>

      <div class="layout-content">
        <div class="content-wrapper">
          <div class="main-list-column">
            <ul class="todo-list">
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
            </ul>
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
        v-if="isCategoryModalOpen"
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
        v-if="isDeleteCategoryConfirmOpen"
        :visible="isDeleteCategoryConfirmOpen"
        :category-name="pendingDeleteCategoryName"
        :task-count="pendingDeleteCategoryTaskCount"
        @close="closeDeleteCategoryConfirm"
        @confirm="confirmDeleteCategory"
      />

      <TaskDetailModal
        v-if="viewTaskData"
        :visible="!!viewTaskData"
        :task="viewTaskData"
        :categories="categories"
        @close="closeTaskDetail"
        @edit="startEditTask"
        @toggle="handleToggleFromDetail"
      />

      <SettingsModal
        v-if="isSettingsModalOpen"
        :visible="isSettingsModalOpen"
        :theme="settingsForm.theme"
        :data-location="settingsForm.dataLocation"
        :data-format="settingsForm.dataFormat"
        :ai-model="settingsForm.aiModel"
        :ai-base-url="settingsForm.aiBaseUrl"
        :ai-api-mode="settingsForm.aiApiMode"
        :ai-endpoint="settingsForm.aiEndpoint"
        :ai-api-key="settingsForm.aiApiKey"
        :default-data-dir-placeholder="defaultDataDirPlaceholder"
        :is-saving="isSavingSettings"
        :error="settingsSaveError"
        :success="settingsSaveSuccess"
        :is-generating-report="isGeneratingReport"
        :report-content="reportContent"
        :report-period="reportPeriod"
        :report-error="reportError"
        @close="closeSettingsModal"
        @save="handleSaveSettings"
        @generate-report="handleGenerateReport"
        @update:theme="settingsForm.theme = $event"
        @update:data-location="settingsForm.dataLocation = $event"
        @update:data-format="settingsForm.dataFormat = $event"
        @update:ai-model="settingsForm.aiModel = $event"
        @update:ai-base-url="settingsForm.aiBaseUrl = $event"
        @update:ai-api-mode="settingsForm.aiApiMode = $event"
        @update:ai-endpoint="settingsForm.aiEndpoint = $event"
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

.storage-error-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 8px 16px;
  margin: 0 24px;
  background: #fef2f2;
  border: 1px solid #fca5a5;
  border-radius: var(--radius-md);
  color: #dc2626;
  font-size: 13px;
}

.storage-error-text {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.storage-error-dismiss {
  flex-shrink: 0;
  border: none;
  background: none;
  color: #dc2626;
  font-size: 18px;
  cursor: pointer;
  padding: 0 4px;
  line-height: 1;
}

.storage-error-dismiss:hover {
  opacity: 0.7;
}
</style>
