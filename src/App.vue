<script setup lang="ts">
import { storeToRefs } from 'pinia';
import { computed, onMounted, ref, watch } from 'vue';
import type { TodoItem as TodoTask } from '@/data/todos';
import StatCard from '@/components/StatCard.vue';
import TodoItem from '@/components/TodoItem.vue';
import TaskTimeline from '@/components/TaskTimeline.vue';
import { useTodoStore } from '@/stores/todo';
import { getDefaultDataDir } from '@/services/storageService';

const todoStore = useTodoStore();
const { filter, selectedCategoryId, categories, visibleTodos, stats, searchQuery, currentSettings } = storeToRefs(todoStore);
const { toggleTodo, updateSettings, updateCategory, updateTodo } = todoStore;

// ─── 应用启动时从持久化存储加载数据 ──────────────────
onMounted(() => {
  todoStore.initialize();
});


const filterOptions = [
  { label: '全部任务', value: 'all', icon: 'M4 6h16M4 12h16M4 18h7' },
  { label: '待处理', value: 'active', icon: 'M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z' },
  { label: '已完成', value: 'completed', icon: 'M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z' },
] as const;

const currentFilterTitle = computed(() => {
  if (selectedCategoryId.value) {
    return categories.value.find(c => c.id === selectedCategoryId.value)?.name || '分类列表';
  }
  return filterOptions.find(o => o.value === filter.value)?.label || '待办列表';
});

const defaultTaskCategoryId = computed(() => categories.value[0]?.id || '');

const formatDisplayDate = (dateString?: string) => {
  if (!dateString) return '—';
  const date = new Date(dateString);
  if (Number.isNaN(date.getTime())) return '—';
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')} ${String(date.getHours()).padStart(2, '0')}:${String(date.getMinutes()).padStart(2, '0')}`;
};

const isModalOpen = ref(false);
const editingTaskId = ref<number | null>(null);
const taskSaveSuccess = ref(false);
let taskSaveSuccessTimer: ReturnType<typeof setTimeout> | null = null;
const newTaskForm = ref({
  title: '',
  categoryId: '',
  priority: 'low' as 'low' | 'medium' | 'high',
  description: ''
});

watch(
  defaultTaskCategoryId,
  (categoryId) => {
    if (!editingTaskId.value && !newTaskForm.value.categoryId && categoryId) {
      newTaskForm.value.categoryId = categoryId;
    }
  },
  { immediate: true }
);

const resetTaskForm = () => {
  editingTaskId.value = null;
  newTaskForm.value = {
    title: '',
    categoryId: defaultTaskCategoryId.value,
    priority: 'low',
    description: ''
  };
};

const onNewTask = () => {
  taskSaveSuccess.value = false;
  resetTaskForm();
  isModalOpen.value = true;
};

const startEditTask = (todo: TodoTask) => {
  taskSaveSuccess.value = false;
  editingTaskId.value = todo.id;
  newTaskForm.value = {
    title: todo.title,
    categoryId: todo.categoryId,
    priority: todo.priority,
    description: todo.description || ''
  };
  closeTaskDetail();
  isModalOpen.value = true;
};

const submitTask = () => {
  if (!newTaskForm.value.title.trim()) return;
  if (!newTaskForm.value.categoryId) return;

  const isEditingTask = editingTaskId.value !== null;

  if (isEditingTask) {
    const taskId = editingTaskId.value;
    if (taskId === null) return;

    updateTodo(
      taskId,
      newTaskForm.value.title.trim(),
      newTaskForm.value.priority,
      newTaskForm.value.categoryId,
      newTaskForm.value.description.trim()
    );

    taskSaveSuccess.value = true;
    if (taskSaveSuccessTimer) clearTimeout(taskSaveSuccessTimer);
    taskSaveSuccessTimer = setTimeout(() => {
      taskSaveSuccess.value = false;
    }, 1800);
  } else {
    todoStore.addTodo(
      newTaskForm.value.title.trim(), 
      newTaskForm.value.priority, 
      newTaskForm.value.categoryId,
      newTaskForm.value.description.trim()
    );
  }

  isModalOpen.value = false;
  resetTaskForm();
};

const isCategoryModalOpen = ref(false);
const presetColors = ['#0A84FF', '#30D158', '#FF9F0A', '#FF453A', '#98989D', '#AF52DE'];
const presetCategoryIcons = ['📁', '💼', '💻', '📚', '🏠', '🛒', '🏃', '💡', '🎯', '✈️', '🎵', '🍽️'];
const editingCategoryId = ref<string | null>(null);
const newCategoryForm = ref({ name: '', color: '#0A84FF', icon: '📁' });

const resetCategoryForm = () => {
  editingCategoryId.value = null;
  newCategoryForm.value = { name: '', color: '#0A84FF', icon: '📁' };
};

const openCategoryModal = () => isCategoryModalOpen.value = true;
const closeCategoryModal = () => {
  isCategoryModalOpen.value = false;
  resetCategoryForm();
};
const selectCategoryIcon = (icon: string) => {
  newCategoryForm.value.icon = icon;
};
const startEditCategory = (categoryId: string) => {
  const target = categories.value.find(category => category.id === categoryId);
  if (!target) return;
  editingCategoryId.value = categoryId;
  newCategoryForm.value = {
    name: target.name,
    color: target.color,
    icon: target.icon,
  };
};
const cancelEditCategory = () => {
  resetCategoryForm();
};

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
  aiApiKey: ''
});

const openSettingsModal = async () => {
  // 同步当前存储类型
  settingsForm.value.dataFormat = currentSettings.value.storageType;
  // 同步当前自定义目录（无则置空，让 placeholder 显示默认路径）
  settingsForm.value.dataLocation = currentSettings.value.dataDir ?? '';
  settingsSaveError.value = null;
  settingsSaveSuccess.value = false;
  // 获取系统默认路径作为占位符
  try {
    defaultDataDirPlaceholder.value = await getDefaultDataDir();
  } catch {
    defaultDataDirPlaceholder.value = '~/Library/Application Support/com.todo.app';
  }
  isSettingsModalOpen.value = true;
};

const closeSettingsModal = () => {
  isSettingsModalOpen.value = false;
};

const saveSettings = async () => {
  if (isSavingSettings.value) return;
  settingsSaveError.value = null;
  settingsSaveSuccess.value = false;
  isSavingSettings.value = true;
  try {
    await updateSettings(settingsForm.value.dataFormat, settingsForm.value.dataLocation);
    settingsSaveSuccess.value = true;
    setTimeout(() => { settingsSaveSuccess.value = false; }, 2000);
  } catch (err) {
    settingsSaveError.value = String(err);
  } finally {
    isSavingSettings.value = false;
  }
};

const viewTaskData = ref<any>(null);
const openTaskDetail = (todo: any) => viewTaskData.value = todo;
const closeTaskDetail = () => viewTaskData.value = null;

const getCategoryForTask = (categoryId: string) => categories.value.find(c => c.id === categoryId);

const submitCategory = () => {
  if (!newCategoryForm.value.name.trim() || !newCategoryForm.value.icon.trim()) return;

  if (editingCategoryId.value) {
    updateCategory(
      editingCategoryId.value,
      newCategoryForm.value.name.trim(),
      newCategoryForm.value.color,
      newCategoryForm.value.icon.trim()
    );
  } else {
    todoStore.addCategory(
      newCategoryForm.value.name.trim(),
      newCategoryForm.value.color,
      newCategoryForm.value.icon.trim()
    );
  }

  resetCategoryForm();
};

const selectCategoryFilter = (id: string | null) => {
  selectedCategoryId.value = id;
  if (id) filter.value = 'all'; // Reset basic filter when a category is selected
};

const closeModal = () => {
  isModalOpen.value = false;
  resetTaskForm();
};
</script>

<template>
  <div class="layout-app">
    <!-- 侧边栏 (Sidebar) -->
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
        <button class="btn-primary new-task-btn" @click="onNewTask">
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
          @click="filter = option.value; selectCategoryFilter(null)"
        >
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path :d="option.icon"></path>
          </svg>
          {{ option.label }}
          <span class="nav-count" v-if="option.value === 'all'">{{ stats.total }}</span>
          <span class="nav-count" v-else-if="option.value === 'active'">{{ stats.pending }}</span>
          <span class="nav-count" v-else>{{ stats.completed }}</span>
        </button>
        <div class="nav-divider"></div>
        <div class="nav-section-header">
          <p class="nav-title">自定义分类</p>
          <button class="icon-btn" @click="openCategoryModal" title="管理分类">+</button>
        </div>
        <button
          v-for="cat in categories"
          :key="cat.id"
          class="nav-item"
          :class="{ 'is-active': selectedCategoryId === cat.id }"
          @click="selectCategoryFilter(cat.id)"
        >
          <span class="cat-icon" :style="{ color: cat.color }">{{ cat.icon }}</span>
          {{ cat.name }}
        </button>
      </nav>

      <div class="sidebar-bottom">
        <p class="nav-title">数据统计</p>
        <div class="stats-container">
          <StatCard title="总任务" :value="stats.total" />
          <StatCard title="待办" :value="stats.pending" />
          <StatCard title="已完成" :value="stats.completed" />
        </div>
      </div>
    </aside>

    <!-- 主体内容 (Main Content) -->
    <main class="layout-main">
      <header class="layout-header">
        <div class="header-title">
          <h1>{{ currentFilterTitle }}</h1>
          <p class="header-subtitle">{{ visibleTodos.length }} 个任务</p>
        </div>
        <div class="header-actions">
          <div class="search-box">
            <svg class="search-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="11" cy="11" r="8"></circle>
              <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
            </svg>
            <input v-model="searchQuery" type="text" placeholder="搜索任务..." />
          </div>
          <button class="icon-btn settings-btn" @click="openSettingsModal" title="系统设置" style="margin-left: 16px; background-color: var(--c-bg-surface); width: 38px; height: 38px; border-radius: var(--radius-sm); border: 1px solid var(--c-border); display: flex; align-items: center; justify-content: center;">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="12" cy="12" r="3"></circle>
              <path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-2 2 2 2 0 01-2-2v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83 0 2 2 0 010-2.83l.06-.06a1.65 1.65 0 00.33-1.82 1.65 1.65 0 00-1.51-1H3a2 2 0 01-2-2 2 2 0 012-2h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 010-2.83 2 2 0 012.83 0l.06.06a1.65 1.65 0 001.82.33H9a1.65 1.65 0 001-1.51V3a2 2 0 012-2 2 2 0 012 2v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 0 2 2 0 010 2.83l-.06.06a1.65 1.65 0 00-.33 1.82V9a1.65 1.65 0 001.51 1H21a2 2 0 012 2 2 2 0 01-2 2h-.09a1.65 1.65 0 00-1.51 1z"></path>
            </svg>
          </button>
        </div>
      </header>

      <div class="layout-content">
        <div class="content-wrapper">
          <div class="main-list-column">
            <TransitionGroup name="fade-up" tag="ul" class="todo-list">
              <TodoItem
                v-for="todo in visibleTodos"
                :key="todo.id"
                :todo="todo"
                @toggle="toggleTodo"
                @edit="startEditTask"
                @view="openTaskDetail"
              />

              <!-- 空状态占位 -->
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
          
          <!-- 右侧时间轴 -->
          <aside class="timeline-column">
            <TaskTimeline />
          </aside>
        </div>
      </div>
    </main>

    <!-- 新建任务弹窗 (Modal) -->
    <Teleport to="body">
      <div v-if="taskSaveSuccess" class="floating-toast toast-success">
        ✅ 任务已保存
      </div>
      <Transition name="fade-up">
        <div v-if="isModalOpen" class="modal-overlay" @click.self="closeModal">
          <div class="modal-content">
            <header class="modal-header">
              <h2>{{ editingTaskId !== null ? '编辑任务' : '新建任务' }}</h2>
              <button class="close-btn" @click="closeModal">
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M18 6L6 18M6 6l12 12"></path>
                </svg>
              </button>
            </header>
            <div class="modal-body">
              <div class="form-group">
                <label>任务标题</label>
                <input v-model="newTaskForm.title" type="text" placeholder="准备做什么？" @keyup.enter="submitTask" autofocus />
              </div>
              <div class="form-group">
                <label>详细描述 (可选)</label>
                <textarea v-model="newTaskForm.description" placeholder="添加更多细节..." rows="5"></textarea>
              </div>
              <div class="form-row">
                <div class="form-group">
                  <label>分类</label>
                  <select v-model="newTaskForm.categoryId">
                    <option value="" disabled>请选择分类</option>
                    <option v-for="cat in categories" :key="cat.id" :value="cat.id">
                      {{ cat.icon }} {{ cat.name }}
                    </option>
                  </select>
                </div>
                <div class="form-group">
                  <label>优先级</label>
                  <select v-model="newTaskForm.priority">
                    <option value="low">低 (Low)</option>
                    <option value="medium">中 (Medium)</option>
                    <option value="high">高 (High)</option>
                  </select>
                </div>
              </div>
            </div>
            <footer class="modal-footer">
              <button class="btn-ghost" @click="closeModal">取消</button>
              <button class="btn-primary" @click="submitTask" :disabled="!newTaskForm.title.trim() || !newTaskForm.categoryId">
                {{ editingTaskId !== null ? '保存任务' : '添加任务' }}
              </button>
            </footer>
          </div>
        </div>
      </Transition>
      <Transition name="fade-up">
        <div v-if="isCategoryModalOpen" class="modal-overlay" @click.self="closeCategoryModal">
          <div class="modal-content">
            <header class="modal-header">
              <h2>管理分类</h2>
              <button class="close-btn" @click="closeCategoryModal">
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M18 6L6 18M6 6l12 12"></path>
                </svg>
              </button>
            </header>
            <div class="modal-body">
              <div class="category-list">
                <div v-for="cat in categories" :key="cat.id" class="category-row">
                  <div class="cat-info">
                    <span class="cat-icon-preview" :style="{ backgroundColor: cat.color + '20', color: cat.color }">
                      {{ cat.icon }}
                    </span>
                    <span>{{ cat.name }}</span>
                  </div>
                  <div class="category-actions">
                    <button class="btn-icon-secondary" @click="startEditCategory(cat.id)" title="编辑分类">
                      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M12 20h9"></path>
                        <path d="M16.5 3.5a2.12 2.12 0 113 3L7 19l-4 1 1-4 12.5-12.5z"></path>
                      </svg>
                    </button>
                    <button class="btn-icon-danger" @click="todoStore.deleteCategory(cat.id)" title="删除分类">
                      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M3 6h18M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2M10 11v6M14 11v6"></path>
                      </svg>
                    </button>
                  </div>
                </div>
                <div v-if="categories.length === 0" class="empty-text">无自定义分类</div>
              </div>
              <div class="add-category-box">
                <div class="category-form-header">
                  <h3>{{ editingCategoryId ? '编辑分类' : '新建分类' }}</h3>
                  <button v-if="editingCategoryId" class="btn-text" type="button" @click="cancelEditCategory">取消编辑</button>
                </div>
                <div class="form-row category-create-row">
                  <div class="form-group emoji-picker-group">
                    <label>图标</label>
                    <div class="selected-icon-card" :style="{ backgroundColor: newCategoryForm.color + '20', color: newCategoryForm.color }">
                      <span class="selected-icon-emoji">{{ newCategoryForm.icon }}</span>
                      <span class="selected-icon-label">当前图标</span>
                    </div>
                  </div>
                  <div class="form-group" style="flex: 1;">
                    <label>名称</label>
                    <input v-model="newCategoryForm.name" type="text" placeholder="输入名称..." @keyup.enter="submitCategory" />
                  </div>
                </div>
                <div class="form-group icon-picker-field">
                  <label>选择图标</label>
                  <div class="icon-picker-grid">
                    <button
                      v-for="icon in presetCategoryIcons"
                      :key="icon"
                      type="button"
                      class="icon-choice"
                      :class="{ 'is-active': newCategoryForm.icon === icon }"
                      :style="newCategoryForm.icon === icon ? { backgroundColor: newCategoryForm.color + '20', color: newCategoryForm.color, borderColor: newCategoryForm.color + '55' } : {}"
                      @click="selectCategoryIcon(icon)"
                    >
                      <span>{{ icon }}</span>
                    </button>
                  </div>
                </div>
                <div class="form-group mt-2">
                  <label>主题色</label>
                  <div class="color-picker" :style="{ color: newCategoryForm.color }">
                    <button 
                      v-for="color in presetColors" 
                      :key="color" 
                      type="button"
                      class="color-swatch" 
                      :class="{ 'is-active': newCategoryForm.color === color }"
                      :style="{ backgroundColor: color }"
                      @click="newCategoryForm.color = color"
                    >
                      <svg v-if="newCategoryForm.color === color" class="check-icon-color" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
                        <path d="M5 13l4 4L19 7" stroke-linecap="round" stroke-linejoin="round"/>
                      </svg>
                    </button>
                  </div>
                </div>
                <button class="btn-primary w-full mt-2" @click="submitCategory" :disabled="!newCategoryForm.name.trim()">
                  {{ editingCategoryId ? '保存修改' : '添加分类' }}
                </button>
              </div>
            </div>
          </div>
        </div>
      </Transition>
      <Transition name="fade-up">
        <div v-if="viewTaskData" class="modal-overlay" @click.self="closeTaskDetail">
          <div class="modal-content" style="max-width: 520px;">
            <header class="modal-header">
              <h2>任务详情</h2>
              <button class="close-btn" @click="closeTaskDetail">
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M18 6L6 18M6 6l12 12"></path>
                </svg>
              </button>
            </header>
            <div class="modal-body" style="gap: 20px;">
              <div class="detail-header-block">
                <h1 style="font-size: 1.4rem; color: var(--c-text-primary); margin-bottom: 12px; font-weight: 600; line-height: 1.3">{{ viewTaskData.title }}</h1>
                <div class="meta" style="display: flex; gap: 8px; align-items: center">
                  <span class="pill" v-if="getCategoryForTask(viewTaskData.categoryId)" :style="{ backgroundColor: getCategoryForTask(viewTaskData.categoryId)?.color + '20', color: getCategoryForTask(viewTaskData.categoryId)?.color, padding: '4px 10px', borderRadius: '4px', fontSize: '0.8rem', fontWeight: 600 }">
                    {{ getCategoryForTask(viewTaskData.categoryId)?.icon }} {{ getCategoryForTask(viewTaskData.categoryId)?.name }}
                  </span>
                  <span class="pill" v-else style="background-color: var(--c-bg-sidebar-hover); color: var(--c-text-secondary); padding: 4px 10px; border-radius: 4px">未分类</span>
                  
                  <span class="pill"
                        :style="{
                          backgroundColor: viewTaskData.priority === 'high' ? 'rgba(255,59,48,0.1)' : viewTaskData.priority === 'medium' ? 'rgba(255,149,0,0.1)' : 'rgba(52,199,89,0.1)',
                          color: viewTaskData.priority === 'high' ? 'var(--c-danger)' : viewTaskData.priority === 'medium' ? 'var(--c-warning)' : 'var(--c-success)',
                          padding: '4px 10px', borderRadius: '4px', fontSize: '0.8rem', fontWeight: 600
                        }"
                  >
                    {{ viewTaskData.priority === 'high' ? '高' : viewTaskData.priority === 'medium' ? '中' : '低' }}
                  </span>
                  <span style="margin-left: auto; font-size: 0.85rem; color: var(--c-text-muted)">{{ viewTaskData.dueLabel }}</span>
                </div>
              </div>
              
              <div style="display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 12px;">
                <div style="background-color: var(--c-bg-app); border: 1px solid var(--c-border-light); border-radius: 8px; padding: 14px 16px;">
                  <div style="font-size: 0.78rem; color: var(--c-text-secondary); text-transform: uppercase; font-weight: 600; margin-bottom: 8px;">创建日期</div>
                  <div style="font-size: 0.95rem; color: var(--c-text-primary); font-weight: 500;">{{ formatDisplayDate(viewTaskData.createdAt) }}</div>
                </div>
                <div style="background-color: var(--c-bg-app); border: 1px solid var(--c-border-light); border-radius: 8px; padding: 14px 16px;">
                  <div style="font-size: 0.78rem; color: var(--c-text-secondary); text-transform: uppercase; font-weight: 600; margin-bottom: 8px;">完成日期</div>
                  <div style="font-size: 0.95rem; color: viewTaskData.completedAt ? 'var(--c-success)' : 'var(--c-text-muted)'; font-weight: 500;">{{ formatDisplayDate(viewTaskData.completedAt) }}</div>
                </div>
              </div>
              
              <div class="detail-desc-block" style="background-color: var(--c-bg-app); border: 1px solid var(--c-border-light); border-radius: 8px; padding: 16px; min-height: 240px; max-height: 400px; overflow-y: auto;">
                <h3 style="font-size: 0.85rem; margin-bottom: 12px; color: var(--c-text-secondary); font-weight: 600; text-transform: uppercase">详细描述</h3>
                <p v-if="viewTaskData.description" style="white-space: pre-wrap; font-size: 0.95rem; color: var(--c-text-primary); line-height: 1.6;">{{ viewTaskData.description }}</p>
                <p v-else style="color: var(--c-text-muted); font-size: 0.9rem; font-style: italic;">暂无描述信息...</p>
              </div>
            </div>
            <footer class="modal-footer">
              <div class="task-detail-actions">
                <button class="btn-ghost" @click="startEditTask(viewTaskData)">
                  编辑任务
                </button>
                <button class="btn-primary" v-if="!viewTaskData.completed" @click="toggleTodo(viewTaskData.id); closeTaskDetail()">
                  标记为已完成
                </button>
                <button class="btn-ghost" v-else @click="toggleTodo(viewTaskData.id); closeTaskDetail()">
                  重新激活任务
                </button>
              </div>
            </footer>
          </div>
        </div>
      </Transition>
      <Transition name="fade-up">
        <div v-if="isSettingsModalOpen" class="modal-overlay" @click.self="closeSettingsModal">
          <div class="modal-content" style="max-width: 480px;">
            <header class="modal-header">
              <h2>系统设置</h2>
              <button class="close-btn" @click="closeSettingsModal">
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M18 6L6 18M6 6l12 12"></path>
                </svg>
              </button>
            </header>
            <div class="modal-body" style="gap: 24px; max-height: 65vh; overflow-y: auto;">
              <!-- 1. 主题设置 -->
              <div class="settings-section">
                 <h3 style="font-size: 0.85rem; margin-bottom: 12px; color: var(--c-text-secondary); font-weight: 600; text-transform: uppercase">外观体验</h3>
                 <div style="background-color: var(--c-bg-app); border: 1px solid var(--c-border-light); border-radius: 8px; padding: 16px;">
                    <div class="form-group">
                      <label>主题模式</label>
                      <div style="display: flex; gap: 20px; margin-top: 8px;">
                        <label style="display: flex; align-items: center; gap: 6px; cursor: pointer; font-size: 0.95rem; color: var(--c-text-primary);">
                          <input type="radio" v-model="settingsForm.theme" value="light" style="accent-color: var(--c-accent);" />
                          亮色模式
                        </label>
                        <label style="display: flex; align-items: center; gap: 6px; cursor: pointer; font-size: 0.95rem; color: var(--c-text-primary);">
                          <input type="radio" v-model="settingsForm.theme" value="dark" style="accent-color: var(--c-accent);" />
                          深色模式
                        </label>
                      </div>
                    </div>
                 </div>
              </div>

              <!-- 2. 数据管理 -->
              <div class="settings-section">
                 <h3 style="font-size: 0.85rem; margin-bottom: 12px; color: var(--c-text-secondary); font-weight: 600; text-transform: uppercase">数据管理</h3>
                 <div style="background-color: var(--c-bg-app); border: 1px solid var(--c-border-light); border-radius: 8px; padding: 16px; display: flex; flex-direction: column; gap: 16px;">
                    <div class="form-group">
                      <label>数据保存位置</label>
                      <input type="text" v-model="settingsForm.dataLocation" :placeholder="defaultDataDirPlaceholder" style="margin-top: 8px; width: 100%; box-sizing: border-box;" />
                      <span style="font-size: 0.78rem; color: var(--c-text-muted); margin-top: 2px;">留空则使用上方默认目录，支持 ~ 路径</span>
                    </div>
                    <div class="form-group">
                      <label>数据保存格式</label>
                      <div style="display: flex; gap: 20px; margin-top: 8px;">
                        <label style="display: flex; align-items: center; gap: 6px; cursor: pointer; font-size: 0.95rem; color: var(--c-text-primary);">
                          <input type="radio" v-model="settingsForm.dataFormat" value="json" style="accent-color: var(--c-accent);" />
                          以文件形式 (JSON)
                        </label>
                        <label style="display: flex; align-items: center; gap: 6px; cursor: pointer; font-size: 0.95rem; color: var(--c-text-primary);">
                          <input type="radio" v-model="settingsForm.dataFormat" value="sqlite" style="accent-color: var(--c-accent);" />
                          使用 SQLite
                        </label>
                      </div>
                    </div>
                 </div>
              </div>
              
              <!-- 3. AI模型配置 -->
              <div class="settings-section">
                 <h3 style="font-size: 0.85rem; margin-bottom: 12px; color: var(--c-text-secondary); font-weight: 600; text-transform: uppercase">AI 模型配置 (职场助手)</h3>
                 <div style="background-color: var(--c-bg-app); border: 1px solid var(--c-border-light); border-radius: 8px; padding: 16px; display: flex; flex-direction: column; gap: 16px;">
                    <p style="font-size: 0.85rem; color: var(--c-text-muted); margin: 0 0 4px 0; line-height: 1.5;">配置大语言模型后，可根据您完成的待办事项，一键智能生成结构化的周报、月报。</p>
                    <div class="form-group">
                      <label>API Key</label>
                      <input type="password" v-model="settingsForm.aiApiKey" placeholder="点击输入专属授权 API 密钥..." style="margin-top: 8px; width: 100%; box-sizing: border-box;" />
                    </div>
                    <div class="form-group">
                      <label>模型名称</label>
                      <input type="text" v-model="settingsForm.aiModel" placeholder="例如: gpt-4o, gemini-1.5-pro" style="margin-top: 8px; width: 100%; box-sizing: border-box;" />
                    </div>
                    <div class="form-group">
                      <label>请求地址</label>
                      <input type="text" v-model="settingsForm.aiBaseUrl" placeholder="例如: https://api.openai.com/v1" style="margin-top: 8px; width: 100%; box-sizing: border-box;" />
                    </div>
                 </div>
              </div>

              <!-- 4. 关于作者 -->
              <div class="settings-section">
                 <h3 style="font-size: 0.85rem; margin-bottom: 12px; color: var(--c-text-secondary); font-weight: 600; text-transform: uppercase">关于作者</h3>
                 <div style="background-color: var(--c-bg-app); border: 1px solid var(--c-border-light); border-radius: 8px; padding: 16px; display: flex; align-items: flex-start; gap: 16px;">
                    <div style="width: 52px; height: 52px; border-radius: 50%; background-color: var(--c-accent-bg); display: flex; align-items: center; justify-content: center; color: var(--c-accent); font-size: 1.6rem; font-weight: bold; flex-shrink: 0;">
                      L
                    </div>
                    <div>
                      <h4 style="font-size: 1.05rem; color: var(--c-text-primary); margin: 0 0 6px 0;">Lionel</h4>
                      <p style="color: var(--c-text-muted); font-size: 0.85rem; margin: 0; line-height: 1.5;">热爱编码与生活，追求极致的用户体验与设计，致力于打造好用的效率工具。</p>
                      <a href="https://github.com/lionel-coderm" target="_blank" style="color: var(--c-accent); font-size: 0.85rem; text-decoration: none; margin-top: 8px; display: inline-flex; align-items: center; gap: 4px;">
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                          <path d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22"></path>
                        </svg>
                        @lionel GitHub
                      </a>
                    </div>
                 </div>
              </div>
            </div>
            <footer class="modal-footer" style="flex-direction: column; gap: 10px;">
              <!-- 错误提示 -->
              <div v-if="settingsSaveError" style="width: 100%; padding: 10px 14px; background: rgba(255,59,48,0.1); border: 1px solid rgba(255,59,48,0.3); border-radius: 8px; color: var(--c-danger); font-size: 0.85rem; line-height: 1.5;">
                ⚠️ {{ settingsSaveError }}
              </div>
              <!-- 成功提示 -->
              <div v-if="settingsSaveSuccess" style="width: 100%; padding: 10px 14px; background: rgba(48,209,88,0.1); border: 1px solid rgba(48,209,88,0.3); border-radius: 8px; color: var(--c-success); font-size: 0.85rem;">
                ✅ 设置已保存
              </div>
              <div style="display: flex; gap: 12px; width: 100%;">
                <button class="btn-ghost" style="flex: 1" @click="closeSettingsModal">关闭</button>
                <button
                  class="btn-primary"
                  style="flex: 2"
                  @click="saveSettings"
                  :disabled="isSavingSettings"
                >
                  <span v-if="isSavingSettings">保存中...</span>
                  <span v-else>保存设置</span>
                </button>
              </div>
            </footer>
          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<style scoped>
/* Sidebar 内部样式 */
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

/* 主内容区内部样式 */
.layout-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.header-actions {
  display: flex;
  align-items: center;
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
.header-title h1 {
  font-size: 1.8rem;
  font-weight: 700;
  margin-bottom: 4px;
}
.header-subtitle {
  color: var(--c-text-secondary);
  font-size: 0.95rem;
}

.todo-list {
  display: flex;
  flex-direction: column;
  gap: 8px; /* 更紧凑的列表 */
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

/* Modal 样式 */
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background-color: rgba(0, 0, 0, 0.4);
  backdrop-filter: blur(4px);
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
}

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

.modal-content {
  background-color: var(--c-bg-surface);
  border: 1px solid var(--c-border);
  border-radius: var(--radius-lg);
  width: 100%;
  max-width: 440px;
  box-shadow: var(--shadow-float);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.modal-header {
  padding: 20px 24px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-bottom: 1px solid var(--c-border-light);
}

.modal-header h2 {
  font-size: 1.2rem;
  font-weight: 600;
}

.close-btn {
  color: var(--c-text-secondary);
  transition: var(--transition-fast);
}
.close-btn:hover {
  color: var(--c-text-primary);
  transform: scale(1.1);
}

.modal-body {
  padding: 24px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
  flex: 1;
}

.form-group label {
  font-size: 0.85rem;
  font-weight: 500;
  color: var(--c-text-secondary);
}

.form-group input,
.form-group select,
.form-group textarea {
  padding: 10px 12px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--c-border);
  background-color: var(--c-bg-app);
  color: var(--c-text-primary);
  font-size: 0.95rem;
  outline: none;
  transition: var(--transition-fast);
  font-family: inherit;
}

.form-group input:focus,
.form-group select:focus,
.form-group textarea:focus {
  border-color: var(--c-accent);
  box-shadow: 0 0 0 3px var(--c-accent-bg);
}

.form-group textarea {
  resize: vertical;
  min-height: 120px;
  max-height: 400px;
}

.form-row {
  display: flex;
  gap: 16px;
}

.modal-footer {
  padding: 16px 24px;
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  background-color: var(--c-bg-app);
  border-top: 1px solid var(--c-border-light);
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

.modal-footer .btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
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

.category-list {
  display: flex;
  flex-direction: column;
  max-height: 240px;
  overflow-y: auto;
  border-radius: var(--radius-md);
  border: 1px solid var(--c-border-light);
  background-color: var(--c-bg-app);
}

.category-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px;
  background-color: transparent;
  border-radius: 0;
  border: none;
  border-bottom: 1px solid var(--c-border-light);
  transition: background-color 0.2s ease;
}
.category-row:last-child {
  border-bottom: none;
}
.category-row:hover {
  background-color: var(--c-bg-surface);
}

.cat-info {
  display: flex;
  align-items: center;
  gap: 12px;
  font-weight: 500;
}

.cat-icon-preview {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-sm);
  font-size: 1.1rem;
}

.category-actions {
  display: flex;
  align-items: center;
  gap: 6px;
}

.btn-icon-secondary,
.btn-icon-danger {
  background: none;
  border: none;
  color: var(--c-text-muted);
  padding: 6px;
  font-size: 1rem;
  cursor: pointer;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
  opacity: 0.4;
}
.category-row:hover .btn-icon-secondary,
.category-row:hover .btn-icon-danger {
  opacity: 1;
}
.btn-icon-secondary:hover {
  color: var(--c-accent);
  background-color: rgba(10, 132, 255, 0.1);
  transform: scale(1.05);
}
.btn-icon-danger:hover {
  color: var(--c-danger);
  background-color: rgba(255, 59, 48, 0.1);
  transform: scale(1.05);
}

.empty-text {
  text-align: center;
  color: var(--c-text-muted);
  font-size: 0.9rem;
  padding: 20px 0;
}

.add-category-box {
  margin-top: 16px;
  padding-top: 16px;
  border-top: 1px dashed var(--c-border);
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.add-category-box h3 {
  font-size: 0.95rem;
  font-weight: 600;
  color: var(--c-text-secondary);
}

.category-form-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.btn-text {
  background: none;
  border: none;
  color: var(--c-accent);
  font-size: 0.85rem;
  font-weight: 600;
  cursor: pointer;
  padding: 0;
}
.btn-text:hover {
  text-decoration: underline;
}

.emoji-picker-group {
  flex: 0 0 84px !important;
}
.category-create-row {
  align-items: stretch;
}
.selected-icon-card {
  min-height: 88px;
  border-radius: 14px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 6px;
  border: 1px solid transparent;
  transition: var(--transition-fast);
}
.selected-icon-emoji {
  font-size: 1.8rem;
  line-height: 1;
}
.selected-icon-label {
  font-size: 0.72rem;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  opacity: 0.78;
}
.icon-picker-field {
  gap: 10px;
}
.icon-picker-grid {
  display: grid;
  grid-template-columns: repeat(6, minmax(0, 1fr));
  gap: 10px;
}
.icon-choice {
  height: 44px;
  border-radius: 12px;
  border: 1px solid var(--c-border-light);
  background-color: var(--c-bg-app);
  color: var(--c-text-primary);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 1.35rem;
  transition: transform 0.18s ease, border-color 0.18s ease, background-color 0.18s ease, box-shadow 0.18s ease;
}
.icon-choice:hover {
  transform: translateY(-1px);
  border-color: var(--c-border);
}
.icon-choice.is-active {
  box-shadow: 0 8px 18px rgba(10, 132, 255, 0.12);
}
.color-picker {
  display: flex;
  gap: 12px;
  padding: 4px 0;
}

.color-swatch {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  border: none;
  cursor: pointer;
  transition: transform 0.2s, opacity 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  color: white;
}
.color-swatch:hover {
  transform: scale(1.1);
  opacity: 0.9;
}
.color-swatch.is-active {
  transform: scale(1.1);
  box-shadow: 0 0 0 2px var(--c-bg-surface), 0 0 0 3px currentColor;
}
.check-icon-color {
  width: 14px;
  height: 14px;
  stroke: white;
}

.w-full {
  width: 100%;
}
.mt-2 {
  margin-top: 8px;
}
</style>
