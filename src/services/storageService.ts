import { invoke } from '@tauri-apps/api/core';
import type { TodoItem, CategoryItem } from '@/data/todos';

// ─── 类型定义 ──────────────────────────────────────────

export interface AppData {
  todos: TodoItem[];
  categories: CategoryItem[];
}

export interface AppSettings {
  storageType: 'json' | 'sqlite';
  dataDir?: string; // 自定义数据目录，undefined 或空字符串 = 系统默认
}

export type SearchFilter = 'all' | 'active' | 'completed';

export interface SearchTodosParams {
  query: string;
  filter: SearchFilter;
  selectedCategoryId?: string | null;
}

// ─── 存储服务 ──────────────────────────────────────────

/**
 * 从当前存储引擎（JSON 或 SQLite）读取全部数据
 */
export async function loadAppData(): Promise<AppData> {
  return invoke<AppData>('load_app_data');
}

/**
 * 将全部数据写入当前存储引擎
 */
export async function saveAppData(data: AppData): Promise<void> {
  return invoke('save_app_data', { data });
}

/**
 * 读取应用设置
 */
export async function loadSettings(): Promise<AppSettings> {
  return invoke<AppSettings>('load_settings');
}

/**
 * 保存设置（切换存储类型或数据目录时自动迁移数据）
 */
export async function saveSettings(settings: AppSettings, data: AppData): Promise<void> {
  return invoke('save_settings', { settings, data });
}

/**
 * 在 SQLite 模式下搜索任务
 */
export async function searchSqliteTodos(params: SearchTodosParams): Promise<TodoItem[]> {
  return invoke<TodoItem[]>('search_todos', {
    query: params.query,
    filter: params.filter,
    selectedCategoryId: params.selectedCategoryId ?? null,
  });
}

/**
 * 获取系统默认数据目录（用于前端占位符展示）
 */
export async function getDefaultDataDir(): Promise<string> {
  return invoke<string>('get_default_data_dir');
}
