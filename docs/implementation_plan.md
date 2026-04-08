# 将 Priority 优先级字段修改为整数枚举类型的实现排期

目前的 `priority` 是 `'high' | 'medium' | 'low'` 字符串类型。将其变更为数字枚举（如 1=高，2=中，3=低）可以更直观地在前端比较、并在 SQLite 的 `ORDER BY` 中直接实现原生的高效排序。

## User Review Required

> [!WARNING]
> 这个重构涉及到修改底层 SQLite 数据库的存储类型，应用端旧的文本数据 `'high'` 等也会被洗掉。我会通过一条 `UPDATE` 语句将旧的存储文本映射为数字。您需要确认这是否符合预期。

## Proposed Changes

### 数据层 (Frontend Data & Types)

#### [MODIFY] `src/data/todos.ts`
- 新增：
  ```typescript
  export enum TaskPriority {
    High = 1,
    Medium = 2,
    Low = 3
  }
  ```
- 替换 `TodoItem` 里的 `priority` 的类型为 `TaskPriority`。
- 修改 `starterTodos` 中的示例数据将其替换为 `TaskPriority.High` 等等枚举值。

#### [MODIFY] `src/stores/todo.ts`
- 修改 `applySortOrder`：原本针对文本的映射函数简化，直接变成 `a.priority - b.priority` 比较即可（因为 1, 2, 3 本身在数字大小上就是按高到底）。
- 确保在新增/更新等函数内引用的字符串改用新的 `TaskPriority`。
- 调用 `searchTodos` 接口时把当前的 `sortOrder.value` 传进 SQLite 后端端。

#### [MODIFY] `src/services/storageService.ts`
- `SearchTodosParams` 和 `searchSqliteTodos` 增加 `sortOrder: string` 的传递映射。

---

### 后端层 (Tauri Rust & SQLite)

#### [MODIFY] `src-tauri/src/models.rs`
- 修改 `TodoItem` 里的 `priority` 字段，将其类型从 `String` 改为 `i32`。
- 修改 `SearchTodosQuery` 添加 `pub sort_order: Option<String>` 参数接收过滤条件。

#### [MODIFY] `src-tauri/src/storage/sqlite_storage.rs`
- **数据库迁移**：由于之前数据库里存的可能是 `"high"`、`"low"`，因此在 `migrate` 方法中执行三条热更新命令，用来平滑将已有表里的旧文本升级成新的整型：
  ```sql
  UPDATE todos SET priority = 1 WHERE priority = 'high';
  UPDATE todos SET priority = 2 WHERE priority = 'medium';
  UPDATE todos SET priority = 3 WHERE priority = 'low';
  ```
- **写入与读取**：修改所有 `row.get(3)` 等解析方法映射到 `i32` 格式进行操作，并更新 `CREATE TABLE IF NOT EXISTS` 将其声明为 `INTEGER`。
- **SQLite 读取查询**：
  在 `search` 方法的 `ORDER BY` 里根据传入的 `sort_order` 参数（比如 `?5`），做原生条件排序：
  ```sql
  ORDER BY
      ...,
      CASE WHEN ?5 = 'priority' THEN priority END ASC, -- 1=high, 2=medium, 3=low，ASC正好最高优先级在前
      id DESC
  ```

---

### UI 组件层 (Vue Presentation)

#### [MODIFY] `src/App.vue`
- 在 `setup` 引入枚举：`import { TaskPriority } from '@/data/todos';`，暴露回模板。
- 将原本判断 `priority === 'high'` 替换为 `priority === TaskPriority.High`。
- `newTaskForm` / `<select>` 替换其 `value` 绑定值为 1/ 2/ 3 等数字。

#### [MODIFY] `src/components/TodoItem.vue`
- 在 `setup` 同样引入 `TaskPriority`，并把模板上显示不同颜色的类名或内联样式逻辑改为识别枚举状态。

## 验证计划 (Verification Plan)
- 编译并进入应用，测试原本带有 `priority="high"` 的旧数据是否会报错或转换成了正确的颜色和标签。
- 切回 SQLite 模式，创建一条“低”优先级的任务，然后点击表头的【排序-优先级】，确认列表数据会重新拉取，且 SQLite 确实按照 `priority = 1, 2, 3` 将重要任务排列在列表的最顶端。
