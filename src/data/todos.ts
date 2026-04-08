export interface CategoryItem {
  id: string;
  name: string;
  color: string;
  icon: string;
}

export enum TaskPriority {
  High = 1,
  Medium = 2,
  Low = 3
}

export interface PriorityMeta {
  label: string;
  enLabel: 'High' | 'Medium' | 'Low';
  key: 'high' | 'medium' | 'low';
  color: string;
  bgColor: string;
}

export const PRIORITY_META: Record<TaskPriority, PriorityMeta> = {
  [TaskPriority.High]: {
    label: '高',
    enLabel: 'High',
    key: 'high',
    color: 'var(--c-danger)',
    bgColor: 'rgba(255, 59, 48, 0.1)',
  },
  [TaskPriority.Medium]: {
    label: '中',
    enLabel: 'Medium',
    key: 'medium',
    color: 'var(--c-warning)',
    bgColor: 'rgba(255, 149, 0, 0.1)',
  },
  [TaskPriority.Low]: {
    label: '低',
    enLabel: 'Low',
    key: 'low',
    color: 'var(--c-success)',
    bgColor: 'rgba(52, 199, 89, 0.1)',
  },
};

export const PRIORITY_LEVELS: TaskPriority[] = [
  TaskPriority.High,
  TaskPriority.Medium,
  TaskPriority.Low,
];

export function isTaskPriority(value: unknown): value is TaskPriority {
  return PRIORITY_LEVELS.includes(value as TaskPriority);
}

export function getPriorityMeta(priority: unknown): PriorityMeta {
  if (isTaskPriority(priority)) {
    return PRIORITY_META[priority];
  }
  return PRIORITY_META[TaskPriority.Low];
}

export interface TodoItem {
  id: number;
  title: string;
  completed: boolean;
  priority: TaskPriority;
  createdAt: string;
  completedAt?: string;
  categoryId: string;
  description?: string;
  isDeleted?: boolean;
  deletedAt?: string;
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
    priority: TaskPriority.High,
    createdAt: '2026-04-06T09:30:00.000Z',
    completedAt: '2026-04-07T18:20:00.000Z',
    categoryId: 'cat-1',
    description: '分离所有的组件和数据模型，确保架构清晰',
  },
  {
    id: 2,
    title: '设计本地任务存储方案',
    completed: false,
    priority: TaskPriority.Medium,
    createdAt: '2026-04-08T09:00:00.000Z',
    categoryId: 'cat-1',
  },
  {
    id: 3,
    title: '补充桌面端交互细节',
    completed: false,
    priority: TaskPriority.Low,
    createdAt: '2026-04-08T10:30:00.000Z',
    categoryId: 'cat-3',
  },
];
