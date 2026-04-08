import type { CategoryItem, TodoItem } from '@/data/todos';
import {
  addSqliteCategory,
  addSqliteTodo,
  deleteSqliteCategory,
  deleteSqliteTodo,
  toggleSqliteTodo,
  updateSqliteCategory,
  updateSqliteTodo,
  type CreateTodoInput,
} from '@/services/storageService';

type TodoPriorityValue = TodoItem['priority'];

export interface TodoMutationStrategy {
  toggleTodo(id: number): Promise<void>;
  addTodo(
    title: string,
    priority: TodoPriorityValue,
    categoryId: string,
    description?: string
  ): Promise<void>;
  updateTodo(
    id: number,
    title: string,
    priority: TodoPriorityValue,
    categoryId: string,
    description?: string
  ): Promise<void>;
  addCategory(name: string, color: string, icon: string): Promise<string>;
  updateCategory(id: string, name: string, color: string, icon: string): Promise<void>;
  deleteCategory(id: string): Promise<void>;
  moveToTrash(id: number): Promise<void>;
  restoreTodo(id: number): Promise<void>;
  permanentlyDeleteTodo(id: number): Promise<void>;
  clearTrash(): Promise<void>;
}

export interface TodoMutationContext {
  getTodos(): TodoItem[];
  setTodos(next: TodoItem[]): void;
  getCategories(): CategoryItem[];
  setCategories(next: CategoryItem[]): void;
  getSelectedCategoryId(): string | null;
  setSelectedCategoryId(next: string | null): void;
  normalizeTodo(todo: TodoItem): TodoItem;
  persistData(): void;
}

function findTodoById(ctx: TodoMutationContext, id: number): TodoItem | undefined {
  return ctx.getTodos().find((item) => item.id === id);
}

function findTodoIndex(ctx: TodoMutationContext, id: number): number {
  return ctx.getTodos().findIndex((item) => item.id === id);
}

function upsertTodoLocally(ctx: TodoMutationContext, todo: TodoItem) {
  const normalized = ctx.normalizeTodo(todo);
  const target = findTodoById(ctx, normalized.id);

  if (!target) {
    ctx.getTodos().unshift(normalized);
    return;
  }

  Object.assign(target, normalized);
}

function replaceTodoLocally(ctx: TodoMutationContext, currentId: number, todo: TodoItem): boolean {
  const index = findTodoIndex(ctx, currentId);
  if (index === -1) {
    return false;
  }

  ctx.getTodos()[index] = ctx.normalizeTodo(todo);
  return true;
}

function buildTodoUpdatePayload(
  ctx: TodoMutationContext,
  target: TodoItem,
  patch: Partial<TodoItem>
): TodoItem {
  return ctx.normalizeTodo({
    ...target,
    ...patch,
  });
}

function removeCategoryFromLocalState(ctx: TodoMutationContext, id: string) {
  ctx.setTodos(ctx.getTodos().filter((todo) => todo.categoryId !== id));
  ctx.setCategories(ctx.getCategories().filter((category) => category.id !== id));
  if (ctx.getSelectedCategoryId() === id) {
    ctx.setSelectedCategoryId(null);
  }
}

function removeTodoFromLocalState(ctx: TodoMutationContext, id: number): boolean {
  const normalizedId = Number(id);
  if (Number.isNaN(normalizedId)) return false;

  const todos = ctx.getTodos();
  const index = todos.findIndex((item) => Number(item.id) === normalizedId);
  if (index === -1) return false;

  todos.splice(index, 1);
  return true;
}

function nextTodoId(ctx: TodoMutationContext): number {
  const todos = ctx.getTodos();
  return todos.length > 0
    ? Math.max(...todos.map((todo) => todo.id)) + 1
    : 1;
}

function nextOptimisticTodoId(ctx: TodoMutationContext): number {
  const minId = ctx.getTodos().reduce((currentMin, todo) => Math.min(currentMin, todo.id), 0);
  return minId <= 0 ? minId - 1 : -1;
}

function createCategory(name: string, color: string, icon: string): CategoryItem {
  return {
    id: `cat-${Date.now()}`,
    name,
    color,
    icon,
  };
}

function createJsonMutationStrategy(ctx: TodoMutationContext): TodoMutationStrategy {
  return {
    async toggleTodo(id: number) {
      const target = findTodoById(ctx, id);
      if (!target) return;

      const nextCompleted = !target.completed;
      target.completed = nextCompleted;
      target.completedAt = nextCompleted ? new Date().toISOString() : undefined;
      ctx.persistData();
    },

    async addTodo(
      title: string,
      priority: TodoPriorityValue,
      categoryId: string,
      description?: string
    ) {
      ctx.getTodos().unshift({
        id: nextTodoId(ctx),
        title,
        completed: false,
        priority,
        createdAt: new Date().toISOString(),
        categoryId,
        description,
      });
      ctx.persistData();
    },

    async updateTodo(
      id: number,
      title: string,
      priority: TodoPriorityValue,
      categoryId: string,
      description?: string
    ) {
      const target = findTodoById(ctx, id);
      if (!target) return;

      target.title = title;
      target.priority = priority;
      target.categoryId = categoryId;
      target.description = description;
      ctx.persistData();
    },

    async addCategory(name: string, color: string, icon: string) {
      const category = createCategory(name, color, icon);
      ctx.getCategories().push(category);
      ctx.persistData();
      return category.id;
    },

    async updateCategory(id: string, name: string, color: string, icon: string) {
      const target = ctx.getCategories().find((category) => category.id === id);
      if (!target) return;

      target.name = name;
      target.color = color;
      target.icon = icon;
      ctx.persistData();
    },

    async deleteCategory(id: string) {
      removeCategoryFromLocalState(ctx, id);
      ctx.persistData();
    },

    async moveToTrash(id: number) {
      const target = findTodoById(ctx, id);
      if (!target) return;

      target.isDeleted = true;
      target.deletedAt = new Date().toISOString();
      ctx.persistData();
    },

    async restoreTodo(id: number) {
      const target = findTodoById(ctx, id);
      if (!target) return;

      target.isDeleted = false;
      target.deletedAt = undefined;
      ctx.persistData();
    },

    async permanentlyDeleteTodo(id: number) {
      const removed = removeTodoFromLocalState(ctx, id);
      if (!removed) return;
      ctx.persistData();
    },

    async clearTrash() {
      ctx.setTodos(ctx.getTodos().filter((item) => !item.isDeleted));
      ctx.persistData();
    },
  };
}

function createSqliteMutationStrategy(ctx: TodoMutationContext): TodoMutationStrategy {
  return {
    async toggleTodo(id: number) {
      const target = findTodoById(ctx, id);
      if (!target) return;

      const updated = await toggleSqliteTodo(id);
      upsertTodoLocally(ctx, updated);
    },

    async addTodo(
      title: string,
      priority: TodoPriorityValue,
      categoryId: string,
      description?: string
    ) {
      const optimisticTodo = ctx.normalizeTodo({
        id: nextOptimisticTodoId(ctx),
        title,
        completed: false,
        priority,
        createdAt: new Date().toISOString(),
        categoryId,
        description,
      });
      ctx.getTodos().unshift(optimisticTodo);

      const input: CreateTodoInput = {
        title,
        priority,
        categoryId,
        description,
        createdAt: optimisticTodo.createdAt,
      };
      try {
        const created = await addSqliteTodo(input);
        const replaced = replaceTodoLocally(ctx, optimisticTodo.id, created);
        if (!replaced) {
          upsertTodoLocally(ctx, created);
        }
      } catch (error) {
        removeTodoFromLocalState(ctx, optimisticTodo.id);
        throw error;
      }
    },

    async updateTodo(
      id: number,
      title: string,
      priority: TodoPriorityValue,
      categoryId: string,
      description?: string
    ) {
      const target = findTodoById(ctx, id);
      if (!target) return;

      const payload = buildTodoUpdatePayload(ctx, target, {
        title,
        priority,
        categoryId,
        description,
      });
      const updated = await updateSqliteTodo(payload);
      upsertTodoLocally(ctx, updated);
    },

    async addCategory(name: string, color: string, icon: string) {
      const category = createCategory(name, color, icon);
      await addSqliteCategory(category);
      ctx.getCategories().push(category);
      return category.id;
    },

    async updateCategory(id: string, name: string, color: string, icon: string) {
      const target = ctx.getCategories().find((category) => category.id === id);
      if (!target) return;

      const payload: CategoryItem = { id, name, color, icon };
      await updateSqliteCategory(payload);
      Object.assign(target, payload);
    },

    async deleteCategory(id: string) {
      await deleteSqliteCategory(id);
      removeCategoryFromLocalState(ctx, id);
    },

    async moveToTrash(id: number) {
      const target = findTodoById(ctx, id);
      if (!target) return;

      const payload = buildTodoUpdatePayload(ctx, target, {
        isDeleted: true,
        deletedAt: new Date().toISOString(),
      });
      const updated = await updateSqliteTodo(payload);
      upsertTodoLocally(ctx, updated);
    },

    async restoreTodo(id: number) {
      const target = findTodoById(ctx, id);
      if (!target) return;

      const payload = buildTodoUpdatePayload(ctx, target, {
        isDeleted: false,
        deletedAt: undefined,
      });
      const updated = await updateSqliteTodo(payload);
      upsertTodoLocally(ctx, updated);
    },

    async permanentlyDeleteTodo(id: number) {
      const normalizedId = Number(id);
      if (Number.isNaN(normalizedId)) return;

      await deleteSqliteTodo(normalizedId);
      removeTodoFromLocalState(ctx, normalizedId);
    },

    async clearTrash() {
      const trashIds = ctx.getTodos()
        .filter((item) => item.isDeleted)
        .map((item) => item.id);

      for (const id of trashIds) {
        await deleteSqliteTodo(id);
      }
      ctx.setTodos(ctx.getTodos().filter((item) => !item.isDeleted));
    },
  };
}

export function createTodoMutationStrategies(ctx: TodoMutationContext): Record<'json' | 'sqlite', TodoMutationStrategy> {
  return {
    json: createJsonMutationStrategy(ctx),
    sqlite: createSqliteMutationStrategy(ctx),
  };
}
