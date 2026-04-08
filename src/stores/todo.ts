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
import { searchTodos } from '@/services/searchService';
import {
  addSqliteCategory,
  addSqliteTodo,
  deleteSqliteCategory,
  deleteSqliteTodo,
  loadAppData,
  saveAppData,
  loadSettings,
  saveSettings,
  toggleSqliteTodo,
  updateSqliteCategory,
  updateSqliteTodo,
  type AppSettings,
  type CreateTodoInput,
  type SearchFilter,
} from '@/services/storageService';

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

// ─── Store ─────────────────────────────────────────────

export const useTodoStore = defineStore('todo', () => {
  let saveTimer: ReturnType<typeof setTimeout> | null = null;
  let searchTimer: ReturnType<typeof setTimeout> | null = null;

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

  onScopeDispose(() => {
    if (saveTimer !== null) {
      clearTimeout(saveTimer);
      saveTimer = null;
    }
    if (searchTimer !== null) {
      clearTimeout(searchTimer);
      searchTimer = null;
    }
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
  let activeSearchToken = 0;

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

  async function refreshVisibleTodosFromMemory() {
    try {
      const result = await searchTodos({
        storageType: 'json',
        todos: todos.value,
        categories: categories.value,
        query: searchQuery.value,
        filter: filter.value,
        selectedCategoryId: selectedCategoryId.value,
        sortOrder: sortOrder.value,
      });
      visibleTodos.value = applySortOrder(normalizeTodos(result));
    } catch (err) {
      storageError.value = String(err);
      visibleTodos.value = [];
    }
  }

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
        sortOrder: sortOrder.value,
      });

      if (searchToken !== activeSearchToken) return;
      visibleTodos.value = applySortOrder(normalizeTodos(result));
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
    [filter, selectedCategoryId, searchQuery, sortOrder, () => currentSettings.value.storageType],
    () => {
      if (currentSettings.value.storageType === 'sqlite') {
        debounceSearch(() => {
          void refreshVisibleTodos();
        });
        return;
      }

      void refreshVisibleTodos();
    },
  );

  watch(
    [todos, categories],
    () => {
      if (currentSettings.value.storageType === 'sqlite') {
        // SQLite 场景先用内存数据刷新界面，避免写入前查询旧库造成“点击无效果”。
        void refreshVisibleTodosFromMemory();
        return;
      }

      void refreshVisibleTodos();
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
      saveAppData({ todos: todos.value, categories: categories.value })
        .then(() => {
          // SQLite 模式下列表来自数据库搜索，保存后需要再拉一次以避免显示旧结果。
          if (currentSettings.value.storageType === 'sqlite') {
            return refreshVisibleTodos();
          }
          return undefined;
        })
        .catch((err) => {
          storageError.value = String(err);
        });
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
        const defaultData = createDefaultAppData();
        todos.value = defaultData.todos;
        categories.value = defaultData.categories;
        await saveAppData({ todos: todos.value, categories: categories.value });
      }

      await refreshVisibleTodos();
    } catch (err) {
      storageError.value = String(err);
      // 降级为默认数据
      const defaultData = createDefaultAppData();
      todos.value = defaultData.todos;
      categories.value = defaultData.categories;
      visibleTodos.value = defaultData.todos;
    } finally {
      isLoading.value = false;
    }
  }

  // ─── 变更操作 ─────────────────────────────────────

  async function toggleTodo(id: number) {
    const target = todos.value.find((item) => item.id === id);
    if (!target) return;

    if (currentSettings.value.storageType === 'sqlite') {
      try {
        const updated = normalizeTodo(await toggleSqliteTodo(id));
        Object.assign(target, updated);
      } catch (err) {
        storageError.value = String(err);
      }
      return;
    }

    const nextCompleted = !target.completed;
    target.completed = nextCompleted;
    target.completedAt = nextCompleted ? new Date().toISOString() : undefined;
    persistData();
  }

  async function addTodo(
    title: string,
    priority: TodoItem['priority'],
    categoryId: string,
    description?: string
  ) {
    const nowIso = new Date().toISOString();

    if (currentSettings.value.storageType === 'sqlite') {
      const input: CreateTodoInput = {
        title,
        priority,
        categoryId,
        description,
        createdAt: nowIso,
      };
      try {
        const created = normalizeTodo(await addSqliteTodo(input));
        todos.value.unshift(created);
      } catch (err) {
        storageError.value = String(err);
      }
      return;
    }

    const newId =
      todos.value.length > 0 ? Math.max(...todos.value.map((t) => t.id)) + 1 : 1;

    todos.value.unshift({
      id: newId,
      title,
      completed: false,
      priority,
      createdAt: nowIso,
      categoryId,
      description,
    });
    persistData();
  }

  async function updateTodo(
    id: number,
    title: string,
    priority: TodoItem['priority'],
    categoryId: string,
    description?: string
  ) {
    const target = todos.value.find((item) => item.id === id);
    if (!target) return;

    if (currentSettings.value.storageType === 'sqlite') {
      const updatePayload: TodoItem = normalizeTodo({
        ...target,
        title,
        priority,
        categoryId,
        description,
      });

      try {
        const updated = normalizeTodo(await updateSqliteTodo(updatePayload));
        Object.assign(target, updated);
      } catch (err) {
        storageError.value = String(err);
      }
      return;
    }

    target.title = title;
    target.priority = priority;
    target.categoryId = categoryId;
    target.description = description;
    persistData();
  }

  async function addCategory(name: string, color: string, icon: string) {
    const newId = 'cat-' + Date.now();
    const newCategory: CategoryItem = { id: newId, name, color, icon };

    if (currentSettings.value.storageType === 'sqlite') {
      try {
        await addSqliteCategory(newCategory);
        categories.value.push(newCategory);
      } catch (err) {
        storageError.value = String(err);
      }
      return newId;
    }

    categories.value.push(newCategory);
    persistData();
    return newId;
  }

  async function updateCategory(id: string, name: string, color: string, icon: string) {
    const target = categories.value.find((category) => category.id === id);
    if (!target) return;

    if (currentSettings.value.storageType === 'sqlite') {
      const updatePayload: CategoryItem = {
        id,
        name,
        color,
        icon,
      };
      try {
        await updateSqliteCategory(updatePayload);
        Object.assign(target, updatePayload);
      } catch (err) {
        storageError.value = String(err);
      }
      return;
    }

    target.name = name;
    target.color = color;
    target.icon = icon;
    persistData();
  }

  async function deleteCategory(id: string) {
    if (currentSettings.value.storageType === 'sqlite') {
      try {
        await deleteSqliteCategory(id);
      } catch (err) {
        storageError.value = String(err);
        return;
      }
    }

    todos.value = todos.value.filter((t) => t.categoryId !== id);
    categories.value = categories.value.filter((c) => c.id !== id);
    if (selectedCategoryId.value === id) {
      selectedCategoryId.value = null;
    }

    if (currentSettings.value.storageType !== 'sqlite') {
      persistData();
    }
  }

  async function moveToTrash(id: number) {
    const target = todos.value.find((item) => item.id === id);
    if (!target) return;

    if (currentSettings.value.storageType === 'sqlite') {
      const updatePayload: TodoItem = normalizeTodo({
        ...target,
        isDeleted: true,
        deletedAt: new Date().toISOString(),
      });
      try {
        const updated = normalizeTodo(await updateSqliteTodo(updatePayload));
        Object.assign(target, updated);
      } catch (err) {
        storageError.value = String(err);
      }
      return;
    }

    target.isDeleted = true;
    target.deletedAt = new Date().toISOString();
    persistData();
  }

  async function restoreTodo(id: number) {
    const target = todos.value.find((item) => item.id === id);
    if (!target) return;

    if (currentSettings.value.storageType === 'sqlite') {
      const updatePayload: TodoItem = normalizeTodo({
        ...target,
        isDeleted: false,
        deletedAt: undefined,
      });
      try {
        const updated = normalizeTodo(await updateSqliteTodo(updatePayload));
        Object.assign(target, updated);
      } catch (err) {
        storageError.value = String(err);
      }
      return;
    }

    target.isDeleted = false;
    target.deletedAt = undefined;
    persistData();
  }

  async function permanentlyDeleteTodo(id: number) {
    const normalizedId = Number(id);
    if (Number.isNaN(normalizedId)) return;

    if (currentSettings.value.storageType === 'sqlite') {
      try {
        await deleteSqliteTodo(normalizedId);
      } catch (err) {
        storageError.value = String(err);
        return;
      }
    }

    const index = todos.value.findIndex((item) => Number(item.id) === normalizedId);
    if (index !== -1) {
      todos.value.splice(index, 1);
      if (currentSettings.value.storageType !== 'sqlite') {
        persistData();
      }
    }
  }

  async function clearTrash() {
    if (currentSettings.value.storageType === 'sqlite') {
      const trashIds = todos.value
        .filter((item) => item.isDeleted)
        .map((item) => item.id);
      try {
        for (const id of trashIds) {
          await deleteSqliteTodo(id);
        }
        todos.value = todos.value.filter((item) => !item.isDeleted);
      } catch (err) {
        storageError.value = String(err);
      }
      return;
    }

    todos.value = todos.value.filter((item) => !item.isDeleted);
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
