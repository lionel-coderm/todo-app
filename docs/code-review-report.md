# 代码审查报告

**审查范围**: Priority 文本转整数枚举重构的所有已修改文件
**审查日期**: 2026-04-08
**涉及文件**:

- `src-tauri/src/models.rs`
- `src-tauri/src/storage/sqlite_storage.rs`
- `src/App.vue`
- `src/components/TodoItem.vue`
- `src/data/todos.ts`
- `src/services/searchService.ts`
- `src/services/storageService.ts`
- `src/stores/todo.ts`

---

## 一、严重问题 (Blocker / Critical)

### 1. App.vue 严重超标 — 1467 行 (限制 500 行)

`src/App.vue` 包含脚本 309 行 + 模板 535 行 + 样式 623 行 = **1467 行**，超出上限近 3 倍。

**问题**：违反单一职责原则。一个组件承担了任务 CRUD、分类管理、设置弹窗、任务详情、搜索排序、数据持久化协调等至少 6 项职责。

**建议拆分方案**：

| 新组件 | 职责 | 预计行数 |
|---|---|---|
| `TaskModal.vue` | 新建/编辑任务弹窗 | ~120 |
| `CategoryModal.vue` | 分类管理弹窗 | ~180 |
| `DeleteCategoryConfirm.vue` | 删除分类确认框 | ~40 |
| `TaskDetailModal.vue` | 任务详情查看 | ~100 |
| `SettingsModal.vue` | 系统设置弹窗 | ~180 |
| `Sidebar.vue` | 侧边栏 (过滤器+分类) | ~120 |
| `HeaderBar.vue` | 顶栏 (搜索+排序) | ~60 |
| `App.vue` | 布局容器 + 事件协调 | ~150 |

### 2. 默认数据重复定义 (DRY 违规)

`starterCategories` 和 `starterTodos` 在两处完全重复定义：

- `src/data/todos.ts:27-60` — `starterCategories` + `starterTodos`
- `src/stores/todo.ts:39-72` — `defaultCategories` + `defaultTodos`

**影响**：修改默认数据时需要同步两处，极易遗漏造成不一致。

**修复**：`todo.ts` 应直接导入 `todos.ts` 中的 `starterCategories` 和 `starterTodos`。

---

## 二、重要问题 (Major)

### 3. TodoItem 行映射代码重复 3 次

`sqlite_storage.rs` 中，`row -> TodoItem` 的映射逻辑出现了 3 次：

- `sqlite_storage.rs:282-294` — `search()` 方法
- `sqlite_storage.rs:393-405` — `load()` 方法
- 两处代码完全一致

**修复**：提取为私有函数：

```rust
fn row_to_todo_item(row: &rusqlite::Row) -> rusqlite::Result<TodoItem> {
    Ok(TodoItem {
        id: row.get(0)?,
        title: row.get(1)?,
        completed: row.get::<_, i32>(2)? != 0,
        priority: row.get::<_, i32>(3)?,
        created_at: row.get(4)?,
        completed_at: row.get(5)?,
        category_id: row.get(6)?,
        description: row.get(7)?,
        is_deleted: Some(row.get::<_, i32>(8)? != 0),
        deleted_at: row.get(9)?,
    })
}
```

### 4. TodoItem.vue 模板中 priority 三元表达式过于晦涩

`src/components/TodoItem.vue:77-79`，两行连续的三元嵌套：

```vue
:data-priority="todo.priority === TaskPriority.High ? 'high' : todo.priority === TaskPriority.Medium ? 'medium' : 'low'"
```

```vue
{{ todo.priority === TaskPriority.High ? '高' : todo.priority === TaskPriority.Medium ? '中' : '低' }}
```

**修复**：在 `<script setup>` 中提取 computed 或工具函数：

```typescript
const priorityLabel = computed(() => {
  const map = { [TaskPriority.High]: '高', [TaskPriority.Medium]: '中', [TaskPriority.Low]: '低' }
  return map[props.todo.priority] ?? '低'
})

const priorityKey = computed(() => {
  const map = { [TaskPriority.High]: 'high', [TaskPriority.Medium]: 'medium', [TaskPriority.Low]: 'low' }
  return map[props.todo.priority] ?? 'low'
})
```

模板简化为：

```vue
<span class="pill pill-priority" :data-priority="priorityKey">{{ priorityLabel }}</span>
```

### 5. `viewTaskData` 使用 `any` 类型 — 类型安全缺失

`src/App.vue:273-274`：

```typescript
const viewTaskData = ref<any>(null);
const openTaskDetail = (todo: any) => viewTaskData.value = todo;
```

**修复**：应使用 `TodoItem | null` 类型。

### 6. App.vue 任务详情弹窗中大量 inline style

`src/App.vue:665-701` 区域，几乎所有布局都使用内联 `style="..."`，导致：

- 无法复用 CSS 变量系统
- 难以维护和调整
- 违反样式关注点分离

**修复**：提取为 scoped CSS class。

### 7. SQLite `search()` 排序逻辑不完整

`sqlite_storage.rs:263-273`，ORDER BY 中只处理了 `priority` 排序，`created` 排序完全缺失：

```sql
ORDER BY
    CASE ... END,                    -- 搜索相关度排序
    CASE WHEN ?5 = 'priority' THEN priority END ASC,  -- 优先级排序
    id DESC                          -- 默认排序
```

当 `sortOrder = 'created'` 时，SQLite 端没有按 `created_at DESC` 排序，只能靠前端 `applySortOrder` 补偿。这意味着 SQLite 搜索的分页（如果将来加）会不正确。

**修复**：增加 created 排序分支：

```sql
CASE WHEN ?5 = 'created' THEN created_at END DESC,
CASE WHEN ?5 = 'priority' THEN priority END ASC,
```

---

## 三、一般问题 (Minor)

### 8. 模板中出现 emoji

`src/App.vue:470-471`：

```vue
✅ 任务已保存
```

`src/App.vue:819-820`：

```vue
⚠️ {{ settingsSaveError }}
```

`src/App.vue:823-824`：

```vue
✅ 设置已保存
```

虽然这是 UI 展示而非日志，但考虑到规范中明确禁止使用 emoji/特殊 Unicode 字符，建议确认是否适用于 UI 文案。如果 UI 允许，则无问题。

### 9. `formatDisplayDate` 函数重复定义

`formatDisplayDate` 和 `formatShortDate` 在 `App.vue:37-49` 和 `TodoItem.vue:26-38` 各自定义了一份，逻辑略有差异。

**修复**：提取到 `src/utils/date.ts` 共享。

### 10. SQLite 迁移中 `PRAGMA table_info` 使用 format 拼接

`sqlite_storage.rs:24`：

```rust
let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
```

虽然 `table` 参数来自硬编码的 `"todos"`，但使用 format 拼接 SQL 不是好实践。当前安全，但如果将来 `table` 参数来自外部输入则存在注入风险。建议添加注释标明安全原因。

### 11. `normalizePriority` 可以更简洁

`src/stores/todo.ts:15-20`：

```typescript
function normalizePriority(priority: unknown): TaskPriority {
  if (priority === TaskPriority.High || priority === TaskPriority.Medium || priority === TaskPriority.Low) {
    return priority;
  }
  return TaskPriority.Low;
}
```

**简化**：

```typescript
function normalizePriority(priority: unknown): TaskPriority {
  return Object.values(TaskPriority).includes(priority as number)
    ? (priority as TaskPriority)
    : TaskPriority.Low;
}
```

### 12. `has_column` 和 `column_decl_type` 有重复的 PRAGMA 遍历

`sqlite_storage.rs` 中 `has_column()` 和 `column_decl_type()` 都执行 `PRAGMA table_info` 并遍历行，可以合并为一次调用返回更丰富的结构。

---

## 四、架构建议 (Observation)

### 13. SQLite `save()` 采用全量覆盖策略

`sqlite_storage.rs:417-459`，`save()` 先 `DELETE FROM todos` 再逐行 INSERT。对于大量数据性能差且存在事务风险。

**未来建议**：当数据量增长后，考虑 upsert (INSERT OR REPLACE) 策略，按 id 增量更新。

### 14. Priority 枚举映射分散

优先级标签（"高"/"中"/"低"）和颜色映射散布在 `App.vue`、`TodoItem.vue`、`todo.ts` 等多处。

**建议**：在 `src/data/todos.ts` 中集中定义优先级元数据：

```typescript
export const PRIORITY_META: Record<TaskPriority, { label: string; key: string; color: string }> = {
  [TaskPriority.High]:   { label: '高', key: 'high',   color: 'var(--c-danger)' },
  [TaskPriority.Medium]: { label: '中', key: 'medium', color: 'var(--c-warning)' },
  [TaskPriority.Low]:    { label: '低', key: 'low',    color: 'var(--c-success)' },
}
```

---

## 五、审查汇总

| 级别 | 数量 | 问题编号 |
|---|---|---|
| 严重 (Blocker) | 2 | #1, #2 |
| 重要 (Major) | 5 | #3, #4, #5, #6, #7 |
| 一般 (Minor) | 5 | #8, #9, #10, #11, #12 |
| 观察 (Observation) | 2 | #13, #14 |

**优先修复顺序建议**：

1. **#2** 默认数据重复 — 最易修复，立即解决
2. **#7** SQLite 排序缺失 — 功能缺陷
3. **#1** App.vue 拆分 — 最重要的结构性改进
4. **#3** 行映射重复 — DRY 原则
5. **#4, #5, #6** 模板/类型/样式问题 — 可读性和可维护性
6. 其余问题按优先级逐步处理
