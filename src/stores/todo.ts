import { computed, onScopeDispose, ref, watch } from 'vue';
import { defineStore } from 'pinia';
import {
  isTaskPriority,
  TaskPriority,
  starterCategories,
  starterTodos,
  type TodoItem,
  type CategoryItem,
} from '@/data/todos';
import { filterTodos, searchTodos } from '@/services/searchService';
import {
  loadAppData,
  saveAppData,
  loadSettings,
  saveSettings,
  type AppSettings,
  type SearchFilter,
} from '@/services/storageService';
import {
  createTodoMutationStrategies,
  type TodoMutationStrategy,
} from '@/stores/todoMutationStrategy';

function normalizePriority(priority: unknown): TaskPriority {
  if (isTaskPriority(priority)) {
    return priority;
  }
  return TaskPriority.Low;
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
    priority: normalizePriority(todo.priority),
    isDeleted: todo.isDeleted || false,
  };
}

// ─── 默认初始数据（首次运行 / 无本地存储时） ─────────────

function createDefaultCategories(): CategoryItem[] {
  return starterCategories.map((item) => ({ ...item }));
}

function createDefaultTodos(): TodoItem[] {
  return starterTodos.map((item) => normalizeTodo({ ...item }));
}

function createDefaultAppData(): { todos: TodoItem[]; categories: CategoryItem[] } {
  return {
    todos: createDefaultTodos(),
    categories: createDefaultCategories(),
  };
}

function normalizeTodos(todos: TodoItem[]) {
  return todos.map(normalizeTodo);
}

const APP_CACHE_KEY = 'todo-studio-cache-v1';

interface AppCacheSnapshot {
  todos: TodoItem[];
  categories: CategoryItem[];
  settings: AppSettings;
}

interface IdleDeadlineLike {
  didTimeout: boolean;
  timeRemaining: () => number;
}

type IdleCallbackHandle = number;
type RequestIdleCallbackLike = (
  callback: (deadline: IdleDeadlineLike) => void,
  options?: { timeout: number }
) => IdleCallbackHandle;
type CancelIdleCallbackLike = (handle: IdleCallbackHandle) => void;

function readAppCache(): AppCacheSnapshot | null {
  if (typeof window === 'undefined') {
    return null;
  }

  try {
    const raw = window.localStorage.getItem(APP_CACHE_KEY);
    if (!raw) {
      return null;
    }

    const parsed = JSON.parse(raw) as Partial<AppCacheSnapshot>;
    if (!Array.isArray(parsed.todos) || !Array.isArray(parsed.categories) || !parsed.settings) {
      return null;
    }

    return {
      todos: parsed.todos,
      categories: parsed.categories,
      settings: parsed.settings,
    };
  } catch {
    return null;
  }
}

function writeAppCache(snapshot: AppCacheSnapshot) {
  if (typeof window === 'undefined') {
    return;
  }

  try {
    window.localStorage.setItem(APP_CACHE_KEY, JSON.stringify(snapshot));
  } catch {
    // 忽略缓存写入失败，不能影响主流程
  }
}

// ─── Store ─────────────────────────────────────────────

export const useTodoStore = defineStore('todo', () => {
  let saveTimer: ReturnType<typeof setTimeout> | null = null;
  let searchTimer: ReturnType<typeof setTimeout> | null = null;
  let cacheTimer: ReturnType<typeof setTimeout> | null = null;
  let cacheIdleHandle: IdleCallbackHandle | null = null;

  function debouncedSave(fn: () => void, delay = 500) {
    if (saveTimer !== null) clearTimeout(saveTimer);
    saveTimer = setTimeout(() => {
      saveTimer = null;
      fn();
    }, delay);
  }

  function debounceSearch(fn: () => void, delay = 180) {
    if (searchTimer !== null) clearTimeout(searchTimer);
    searchTimer = setTimeout(() => {
      searchTimer = null;
      fn();
    }, delay);
  }

  function cancelPendingSearch() {
    if (searchTimer !== null) {
      clearTimeout(searchTimer);
      searchTimer = null;
    }
  }

  function flushAppCache() {
    writeAppCache({
      todos: todos.value,
      categories: categories.value,
      settings: currentSettings.value,
    });
  }

  function cancelPendingCacheWrite() {
    if (cacheTimer !== null) {
      clearTimeout(cacheTimer);
      cacheTimer = null;
    }

    if (cacheIdleHandle !== null && typeof window !== 'undefined') {
      const idleWindow = window as Window & {
        cancelIdleCallback?: CancelIdleCallbackLike;
      };
      idleWindow.cancelIdleCallback?.(cacheIdleHandle);
      cacheIdleHandle = null;
    }
  }

  function scheduleCacheWrite(delay = 900) {
    cancelPendingCacheWrite();

    cacheTimer = setTimeout(() => {
      cacheTimer = null;

      if (typeof window === 'undefined') {
        flushAppCache();
        return;
      }

      const idleWindow = window as Window & {
        requestIdleCallback?: RequestIdleCallbackLike;
      };

      if (typeof idleWindow.requestIdleCallback === 'function') {
        cacheIdleHandle = idleWindow.requestIdleCallback(
          () => {
            cacheIdleHandle = null;
            flushAppCache();
          },
          { timeout: 1500 },
        );
        return;
      }

      flushAppCache();
    }, delay);
  }

  onScopeDispose(() => {
    if (saveTimer !== null) {
      clearTimeout(saveTimer);
      saveTimer = null;
    }
    if (searchTimer !== null) {
      clearTimeout(searchTimer);
      searchTimer = null;
    }
    cancelPendingCacheWrite();
  });

  const categories = ref<CategoryItem[]>([]);
  const todos = ref<TodoItem[]>([]);
  const visibleTodos = ref<TodoItem[]>([]);

  const filter = ref<SearchFilter>('all');
  const selectedCategoryId = ref<string | null>(null);
  const searchQuery = ref<string>('');
  const sortOrder = ref<'created' | 'priority'>('created');

  // 存储状态
  const isLoading = ref(false);
  const isSearching = ref(false);
  const storageError = ref<string | null>(null);
  const currentSettings = ref<AppSettings>({ storageType: 'json' });
  const cachedSnapshot = readAppCache();

  if (cachedSnapshot) {
    todos.value = normalizeTodos(cachedSnapshot.todos);
    categories.value = cachedSnapshot.categories;
    currentSettings.value = {
      ...currentSettings.value,
      ...cachedSnapshot.settings,
    };
  }

  function applySortOrder(items: TodoItem[]): TodoItem[] {
    const sorted = [...items];
    if (sortOrder.value === 'priority') {
      sorted.sort((a, b) => {
        // High=1, Medium=2, Low=3。数值越小优先级越高，所以直接 a - b
        const pw = a.priority - b.priority;
        if (pw !== 0) return pw;
        // 优先级相同时，再按创建时间降序
        return new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime();
      });
    } else {
      // 默认：按创建时间降序（最新在前）
      sorted.sort((a, b) => new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime());
    }
    return sorted;
  }

  function refreshVisibleTodosSync() {
    const filtered = filterTodos(todos.value, filter.value, selectedCategoryId.value);
    visibleTodos.value = applySortOrder(filtered);
  }

  async function refreshVisibleTodos() {
    try {
      if (!searchQuery.value.trim()) {
        refreshVisibleTodosSync();
        return;
      }

      const result = await searchTodos({
        storageType: 'json',
        todos: todos.value,
        categories: categories.value,
        query: searchQuery.value,
        filter: filter.value,
        selectedCategoryId: selectedCategoryId.value,
        sortOrder: sortOrder.value,
      });
      visibleTodos.value = applySortOrder(result);
    } catch (err) {
      storageError.value = String(err);
      visibleTodos.value = [];
    }
  }

  refreshVisibleTodosSync();

  watch(
    [filter, selectedCategoryId, sortOrder],
    () => {
      cancelPendingSearch();
      refreshVisibleTodosSync();
    },
  );

  watch(searchQuery, () => {
    if (!searchQuery.value.trim()) {
      cancelPendingSearch();
      refreshVisibleTodosSync();
      return;
    }

    debounceSearch(() => {
      void refreshVisibleTodos();
    });
  });

  watch(() => currentSettings.value.storageType, () => {
    cancelPendingSearch();
    if (!searchQuery.value.trim()) {
      refreshVisibleTodosSync();
      return;
    }
    void refreshVisibleTodos();
  });

  watch(
    [todos, categories],
    () => {
      if (!searchQuery.value.trim()) {
        refreshVisibleTodosSync();
        return;
      }
      void refreshVisibleTodos();
    },
    { deep: true },
  );

  watch(
    [todos, categories, currentSettings],
    () => {
      scheduleCacheWrite();
    },
    { deep: true },
  );

  const stats = computed(() => ({
    total: todos.value.filter((item) => !item.isDeleted).length,
    completed: todos.value.filter((item) => item.completed && !item.isDeleted).length,
    pending: todos.value.filter((item) => !item.completed && !item.isDeleted).length,
    trash: todos.value.filter((item) => item.isDeleted).length,
  }));

  // ─── 持久化辅助 ───────────────────────────────────

  function persistData() {
    if (currentSettings.value.storageType === 'sqlite') {
      return;
    }

    debouncedSave(() => {
      // 防抖期间可能切换到 SQLite，执行前再次检查避免写入错误后端。
      if (currentSettings.value.storageType === 'sqlite') {
        return;
      }

      saveAppData({ todos: todos.value, categories: categories.value }).catch((err) => {
        storageError.value = String(err);
      });
    });
  }

  // ─── 初始化（从存储加载） ──────────────────────────

  async function initialize() {
    isLoading.value = true;
    storageError.value = null;
    try {
      const [settings, data] = await Promise.all([
        loadSettings(),
        loadAppData(),
      ]);
      currentSettings.value = settings;

      if (data.todos.length > 0 || data.categories.length > 0) {
        todos.value = normalizeTodos(data.todos);
        categories.value = data.categories;
      } else {
        // 首次运行，写入默认数据
        const defaultData = createDefaultAppData();
        todos.value = defaultData.todos;
        categories.value = defaultData.categories;
        await saveAppData({ todos: todos.value, categories: categories.value });
      }

      if (!searchQuery.value.trim()) {
        refreshVisibleTodosSync();
      } else {
        await refreshVisibleTodos();
      }
    } catch (err) {
      // 降级为默认数据
      const defaultData = createDefaultAppData();
      todos.value = defaultData.todos;
      categories.value = defaultData.categories;
      refreshVisibleTodosSync();

      try {
        await saveAppData(defaultData);
        storageError.value = null;
        refreshVisibleTodosSync();
      } catch (persistErr) {
        storageError.value = `${String(err)}；默认数据回写失败：${String(persistErr)}`;
      }
    } finally {
      isLoading.value = false;
    }
  }

  // ─── 变更操作 ─────────────────────────────────────

  const mutationStrategies = createTodoMutationStrategies({
    getTodos: () => todos.value,
    setTodos: (next) => {
      todos.value = next;
    },
    getCategories: () => categories.value,
    setCategories: (next) => {
      categories.value = next;
    },
    getSelectedCategoryId: () => selectedCategoryId.value,
    setSelectedCategoryId: (next) => {
      selectedCategoryId.value = next;
    },
    normalizeTodo,
    persistData,
  });

  const currentMutationStrategy = computed<TodoMutationStrategy>(() => {
    return currentSettings.value.storageType === 'sqlite'
      ? mutationStrategies.sqlite
      : mutationStrategies.json;
  });

  async function runMutation(operation: (strategy: TodoMutationStrategy) => Promise<void>) {
    try {
      await operation(currentMutationStrategy.value);
    } catch (err) {
      storageError.value = String(err);
    }
  }

  async function runMutationWithResult<T>(
    operation: (strategy: TodoMutationStrategy) => Promise<T>
  ): Promise<T | undefined> {
    try {
      return await operation(currentMutationStrategy.value);
    } catch (err) {
      storageError.value = String(err);
      return undefined;
    }
  }

  async function toggleTodo(id: number) {
    await runMutation((strategy) => strategy.toggleTodo(id));
  }

  async function addTodo(
    title: string,
    priority: TodoItem['priority'],
    categoryId: string,
    description?: string
  ) {
    await runMutation((strategy) => strategy.addTodo(title, priority, categoryId, description));
  }

  async function updateTodo(
    id: number,
    title: string,
    priority: TodoItem['priority'],
    categoryId: string,
    description?: string
  ) {
    await runMutation((strategy) => strategy.updateTodo(id, title, priority, categoryId, description));
  }

  async function addCategory(name: string, color: string, icon: string) {
    return runMutationWithResult((strategy) => strategy.addCategory(name, color, icon));
  }

  async function updateCategory(id: string, name: string, color: string, icon: string) {
    await runMutation((strategy) => strategy.updateCategory(id, name, color, icon));
  }

  async function deleteCategory(id: string) {
    await runMutation((strategy) => strategy.deleteCategory(id));
  }

  async function moveToTrash(id: number) {
    await runMutation((strategy) => strategy.moveToTrash(id));
  }

  async function restoreTodo(id: number) {
    await runMutation((strategy) => strategy.restoreTodo(id));
  }

  async function permanentlyDeleteTodo(id: number) {
    await runMutation((strategy) => strategy.permanentlyDeleteTodo(id));
  }

  async function clearTrash() {
    await runMutation((strategy) => strategy.clearTrash());
  }

  // ─── 设置更新 ─────────────────────────────────────

  function normalizeOptionalText(value?: string): string | undefined {
    const trimmed = value?.trim();
    return trimmed ? trimmed : undefined;
  }

  async function updateSettings(
    newType: 'json' | 'sqlite',
    newDataDir?: string,
    aiConfig?: {
      aiModel?: string;
      aiBaseUrl?: string;
      aiApiKey?: string;
    },
  ) {
    const newSettings: AppSettings = {
      storageType: newType,
      dataDir: normalizeOptionalText(newDataDir),
      aiModel: normalizeOptionalText(aiConfig?.aiModel),
      aiBaseUrl: normalizeOptionalText(aiConfig?.aiBaseUrl),
      aiApiKey: normalizeOptionalText(aiConfig?.aiApiKey),
    };
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
    sortOrder,
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
    moveToTrash,
    restoreTodo,
    permanentlyDeleteTodo,
    clearTrash,
    updateSettings,
  };
});
