import type { IFuseOptions } from 'fuse.js';
import type { CategoryItem, TodoItem } from '@/data/todos';
import type { SearchFilter } from '@/services/storageService';

export interface SearchTodosParams {
  storageType: 'json' | 'sqlite';
  todos: TodoItem[];
  categories: CategoryItem[];
  query: string;
  filter: SearchFilter;
  selectedCategoryId: string | null;
  sortOrder?: 'created' | 'priority';
}

interface SearchableTodo {
  todo: TodoItem;
  categoryName: string;
}

let fuseConstructorPromise: Promise<(typeof import('fuse.js'))['default']> | null = null;

const fuseOptions: IFuseOptions<SearchableTodo> = {
  includeScore: true,
  threshold: 0.35,
  ignoreLocation: true,
  minMatchCharLength: 1,
  keys: [
    { name: 'todo.title', weight: 0.7 },
    { name: 'todo.description', weight: 0.2 },
    { name: 'categoryName', weight: 0.1 },
  ],
};

function applyFilters(
  todos: TodoItem[],
  filter: SearchFilter,
  selectedCategoryId: string | null,
) {
  let result = todos;

  // 先处理回收站过滤
  if (filter === 'trash') {
    result = result.filter((item) => item.isDeleted === true);
  } else {
    result = result.filter((item) => item.isDeleted !== true);
  }

  // 再处理完成状态过滤（回收站视图不区分完成状态）
  if (filter !== 'trash') {
    if (filter === 'active') {
      result = result.filter((item) => !item.completed);
    } else if (filter === 'completed') {
      result = result.filter((item) => item.completed);
    }
  }

  if (selectedCategoryId) {
    result = result.filter((item) => item.categoryId === selectedCategoryId);
  }

  return result;
}

export function filterTodos(
  todos: TodoItem[],
  filter: SearchFilter,
  selectedCategoryId: string | null,
) {
  return applyFilters(todos, filter, selectedCategoryId);
}

async function getFuseConstructor() {
  if (!fuseConstructorPromise) {
    fuseConstructorPromise = import('fuse.js').then((module) => module.default);
  }
  return fuseConstructorPromise;
}

function searchJsonTodos({
  todos,
  categories,
  query,
  filter,
  selectedCategoryId,
}: Omit<SearchTodosParams, 'storageType'>) {
  const filteredTodos = applyFilters(todos, filter, selectedCategoryId);
  return { filteredTodos, normalizedQuery: query.trim(), categories };
}

async function runFuseSearch({
  filteredTodos,
  categories,
  normalizedQuery,
}: {
  filteredTodos: TodoItem[];
  categories: CategoryItem[];
  normalizedQuery: string;
}) {
  const Fuse = await getFuseConstructor();

  const categoryNameById = new Map(categories.map((category) => [category.id, category.name]));
  const fuse = new Fuse(
    filteredTodos.map((todo) => ({
      todo,
      categoryName: categoryNameById.get(todo.categoryId) || '',
    })),
    fuseOptions,
  );

  return fuse.search(normalizedQuery).map((result) => result.item.todo);
}

export async function searchTodos(params: SearchTodosParams): Promise<TodoItem[]> {
  const localResult = searchJsonTodos(params);
  if (!localResult.normalizedQuery) {
    return localResult.filteredTodos;
  }
  return runFuseSearch(localResult);
}
