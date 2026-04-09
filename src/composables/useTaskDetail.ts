import { ref } from 'vue';
import type { TodoItem } from '@/data/todos';

interface UseTaskDetailOptions {
  toggleTodo: (id: number) => Promise<boolean> | boolean;
}

export function useTaskDetail(options: UseTaskDetailOptions) {
  const { toggleTodo } = options;
  const viewTaskData = ref<TodoItem | null>(null);

  function openTaskDetail(todo: TodoItem) {
    viewTaskData.value = todo;
  }

  function closeTaskDetail() {
    viewTaskData.value = null;
  }

  async function handleToggleFromDetail(id: number) {
    const success = await toggleTodo(id);
    if (!success) return;
    closeTaskDetail();
  }

  return {
    viewTaskData,
    openTaskDetail,
    closeTaskDetail,
    handleToggleFromDetail,
  };
}
