import Fuse from 'fuse.js';
import type { IFuseOptions } from 'fuse.js';
import type { CategoryItem, TodoItem } from '@/data/todos';
import { searchSqliteTodos, type SearchFilter } from '@/services/storageService';

export interface SearchTodosParams {
  storageType: 'json' | 'sqlite';
  todos: TodoItem[];
  categories: CategoryItem[];
  query: string;
  filter: SearchFilter;
  selectedCategoryId: string | null;
}

interface SearchableTodo {
  todo: TodoItem;
  categoryName: string;
}

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

  if (filter === 'active') {
    result = result.filter((item) => !item.completed);
  } else if (filter === 'completed') {
    result = result.filter((item) => item.completed);
  }

  if (selectedCategoryId) {
    result = result.filter((item) => item.categoryId === selectedCategoryId);
  }

  return result;
}

function searchJsonTodos({
  todos,
  categories,
  query,
  filter,
  selectedCategoryId,
}: Omit<SearchTodosParams, 'storageType'>) {
  const filteredTodos = applyFilters(todos, filter, selectedCategoryId);
  const normalizedQuery = query.trim();

  if (!normalizedQuery) {
    return filteredTodos;
  }

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
  if (params.storageType === 'sqlite') {
    return searchSqliteTodos({
      query: params.query,
      filter: params.filter,
      selectedCategoryId: params.selectedCategoryId,
    });
  }

  return searchJsonTodos(params);
}
