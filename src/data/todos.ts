export interface CategoryItem {
  id: string;
  name: string;
  color: string;
  icon: string;
}

export interface TodoItem {
  id: number;
  title: string;
  completed: boolean;
  priority: 'low' | 'medium' | 'high';
  dueLabel: string;
  createdAt: string;
  completedAt?: string;
  categoryId: string;
  description?: string;
}

export const starterCategories: CategoryItem[] = [
  { id: 'cat-1', name: '项目', color: '#0A84FF', icon: '💻' },
  { id: 'cat-2', name: '生活', color: '#30D158', icon: '☕' },
  { id: 'cat-3', name: '学习', color: '#FF9F0A', icon: '📚' },
];

export const starterTodos: TodoItem[] = [
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
