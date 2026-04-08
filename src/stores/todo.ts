import { computed, ref, watch } from 'vue';
import { defineStore } from 'pinia';
import { type TodoItem, type CategoryItem } from '@/data/todos';
import { searchTodos } from '@/services/searchService';
import {
  loadAppData,
  saveAppData,
  loadSettings,
  saveSettings,
  type AppSettings,
  type SearchFilter,
} from '@/services/storageService';

function formatDateLabel(dateString: string) {
  const date = new Date(dateString);
  if (Number.isNaN(date.getTime())) return '未知日期';
  return `${date.getMonth() + 1}月${date.getDate()}日`;
}

function normalizeTodo(todo: TodoItem): TodoItem {
  const createdAt = todo.createdAt || new Date().toISOString();
  const completedAt = todo.completed
    ? todo.completedAt || createdAt
    : undefined;

  return {
    ...todo,
    createdAt,
    completedAt,
    dueLabel: todo.dueLabel || formatDateLabel(createdAt),
  };
}

// ─── 默认初始数据（首次运行 / 无本地存储时） ─────────────

const defaultCategories: CategoryItem[] = [
  { id: 'cat-1', name: '项目', color: '#0A84FF', icon: '💻' },
  { id: 'cat-2', name: '生活', color: '#30D158', icon: '☕' },
  { id: 'cat-3', name: '学习', color: '#FF9F0A', icon: '📚' },
];

const defaultTodos: TodoItem[] = [
  {
    id: 1,
    title: '整理 Tauri + Vue3 工程结构',
    completed: true,
    priority: 'high',
    dueLabel: '4月7日',
    createdAt: '2026-04-06T09:30:00.000Z',
    completedAt: '2026-04-07T18:20:00.000Z',
    categoryId: 'cat-1',
    description: '分离所有的组件和数据模型，确保架构清晰',
  },
  {
    id: 2,
    title: '设计本地任务存储方案',
    completed: false,
    priority: 'medium',
    dueLabel: '4月8日',
    createdAt: '2026-04-08T09:00:00.000Z',
    categoryId: 'cat-1',
  },
  {
    id: 3,
    title: '补充桌面端交互细节',
    completed: false,
    priority: 'low',
    dueLabel: '4月10日',
    createdAt: '2026-04-08T10:30:00.000Z',
    categoryId: 'cat-3',
  },
];

// ─── 防抖工具 ──────────────────────────────────────────

let saveTimer: ReturnType<typeof setTimeout> | null = null;
let searchTimer: ReturnType<typeof setTimeout> | null = null;

function debouncedSave(fn: () => void, delay = 500) {
  if (saveTimer !== null) clearTimeout(saveTimer);
  saveTimer = setTimeout(fn, delay);
}

function debounceSearch(fn: () => void, delay = 180) {
  if (searchTimer !== null) clearTimeout(searchTimer);
  searchTimer = setTimeout(fn, delay);
}

function normalizeTodos(todos: TodoItem[]) {
  return todos.map(normalizeTodo);
}

// ─── Store ─────────────────────────────────────────────

export const useTodoStore = defineStore('todo', () => {
  const categories = ref<CategoryItem[]>([]);
  const todos = ref<TodoItem[]>([]);
  const visibleTodos = ref<TodoItem[]>([]);

  const filter = ref<SearchFilter>('all');
  const selectedCategoryId = ref<string | null>(null);
  const searchQuery = ref<string>('');

  // 存储状态
  const isLoading = ref(false);
  const isSearching = ref(false);
  const storageError = ref<string | null>(null);
  const currentSettings = ref<AppSettings>({ storageType: 'json' });
  let activeSearchToken = 0;

  async function refreshVisibleTodos() {
    const searchToken = ++activeSearchToken;
    const shouldSearchRemotely = currentSettings.value.storageType === 'sqlite';

    if (shouldSearchRemotely) {
      isSearching.value = true;
    }

    try {
      const result = await searchTodos({
        storageType: currentSettings.value.storageType,
        todos: todos.value,
        categories: categories.value,
        query: searchQuery.value,
        filter: filter.value,
        selectedCategoryId: selectedCategoryId.value,
      });

      if (searchToken !== activeSearchToken) return;
      visibleTodos.value = normalizeTodos(result);
    } catch (err) {
      if (searchToken !== activeSearchToken) return;
      storageError.value = String(err);
      visibleTodos.value = [];
    } finally {
      if (searchToken === activeSearchToken) {
        isSearching.value = false;
      }
    }
  }

  watch(
    [todos, categories, filter, selectedCategoryId, searchQuery, () => currentSettings.value.storageType],
    () => {
      if (currentSettings.value.storageType === 'sqlite') {
        debounceSearch(() => {
          void refreshVisibleTodos();
        });
        return;
      }

      void refreshVisibleTodos();
    },
    { deep: true },
  );

  const stats = computed(() => ({
    total: todos.value.length,
    completed: todos.value.filter((item) => item.completed).length,
    pending: todos.value.filter((item) => !item.completed).length,
  }));

  // ─── 持久化辅助 ───────────────────────────────────

  function persistData() {
    debouncedSave(() => {
      saveAppData({ todos: todos.value, categories: categories.value }).catch(
        (err) => (storageError.value = String(err))
      );
    });
  }

  // ─── 初始化（从存储加载） ──────────────────────────

  async function initialize() {
    isLoading.value = true;
    storageError.value = null;
    try {
      // 先读设置，再读数据
      const settings = await loadSettings();
      currentSettings.value = settings;

      const data = await loadAppData();

      if (data.todos.length > 0 || data.categories.length > 0) {
        todos.value = normalizeTodos(data.todos);
        categories.value = data.categories;
      } else {
        // 首次运行，写入默认数据
        todos.value = defaultTodos;
        categories.value = defaultCategories;
        await saveAppData({ todos: todos.value, categories: categories.value });
      }

      await refreshVisibleTodos();
    } catch (err) {
      storageError.value = String(err);
      // 降级为默认数据
      todos.value = defaultTodos;
      categories.value = defaultCategories;
      visibleTodos.value = defaultTodos;
    } finally {
      isLoading.value = false;
    }
  }

  // ─── 变更操作 ─────────────────────────────────────

  function toggleTodo(id: number) {
    const target = todos.value.find((item) => item.id === id);
    if (target) {
      const nextCompleted = !target.completed;
      target.completed = nextCompleted;
      target.completedAt = nextCompleted ? new Date().toISOString() : undefined;
      persistData();
    }
  }

  function addTodo(
    title: string,
    priority: TodoItem['priority'],
    categoryId: string,
    description?: string
  ) {
    const newId =
      todos.value.length > 0 ? Math.max(...todos.value.map((t) => t.id)) + 1 : 1;
    const nowIso = new Date().toISOString();

    todos.value.unshift({
      id: newId,
      title,
      completed: false,
      priority,
      dueLabel: formatDateLabel(nowIso),
      createdAt: nowIso,
      categoryId,
      description,
    });
    persistData();
  }

  function updateTodo(
    id: number,
    title: string,
    priority: TodoItem['priority'],
    categoryId: string,
    description?: string
  ) {
    const target = todos.value.find((item) => item.id === id);
    if (!target) return;
    target.title = title;
    target.priority = priority;
    target.categoryId = categoryId;
    target.description = description;
    persistData();
  }

  function addCategory(name: string, color: string, icon: string) {
    const newId = 'cat-' + Date.now();
    categories.value.push({ id: newId, name, color, icon });
    persistData();
    return newId;
  }

  function updateCategory(id: string, name: string, color: string, icon: string) {
    const target = categories.value.find((category) => category.id === id);
    if (!target) return;
    target.name = name;
    target.color = color;
    target.icon = icon;
    persistData();
  }

  function deleteCategory(id: string) {
    todos.value = todos.value.filter((t) => t.categoryId !== id);
    categories.value = categories.value.filter((c) => c.id !== id);
    if (selectedCategoryId.value === id) {
      selectedCategoryId.value = null;
    }
    persistData();
  }

  // ─── 设置更新 ─────────────────────────────────────

  async function updateSettings(newType: 'json' | 'sqlite', newDataDir?: string) {
    const effectiveDir = newDataDir?.trim() || undefined;
    const newSettings: AppSettings = { storageType: newType, dataDir: effectiveDir };
    try {
      await saveSettings(newSettings, { todos: todos.value, categories: categories.value });
      currentSettings.value = newSettings;
      await refreshVisibleTodos();
    } catch (err) {
      storageError.value = String(err);
      throw err;
    }
  }

  return {
    categories,
    todos,
    filter,
    selectedCategoryId,
    searchQuery,
    visibleTodos,
    stats,
    isLoading,
    isSearching,
    storageError,
    currentSettings,
    initialize,
    toggleTodo,
    addTodo,
    updateTodo,
    addCategory,
    updateCategory,
    deleteCategory,
    updateSettings,
  };
});
