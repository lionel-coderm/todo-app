import { onUnmounted, ref, watch, type Ref } from 'vue';
import { TaskPriority, type TodoItem } from '@/data/todos';

interface TaskFormState {
  title: string;
  categoryId: string;
  priority: TodoItem['priority'];
  description: string;
}

interface UseTaskModalOptions {
  defaultTaskCategoryId: Ref<string>;
  addTodo: (
    title: string,
    priority: TodoItem['priority'],
    categoryId: string,
    description?: string,
  ) => Promise<boolean> | boolean;
  updateTodo: (
    id: number,
    title: string,
    priority: TodoItem['priority'],
    categoryId: string,
    description?: string,
  ) => Promise<boolean> | boolean;
  onBeforeEditTask?: () => void;
}

function createEmptyTaskForm(categoryId = ''): TaskFormState {
  return {
    title: '',
    categoryId,
    priority: TaskPriority.Low,
    description: '',
  };
}

export function useTaskModal(options: UseTaskModalOptions) {
  const { defaultTaskCategoryId, addTodo, updateTodo, onBeforeEditTask } = options;

  const isTaskModalOpen = ref(false);
  const editingTaskId = ref<number | null>(null);
  const taskSaveSuccess = ref(false);
  const taskForm = ref<TaskFormState>(createEmptyTaskForm(defaultTaskCategoryId.value));

  let taskSaveSuccessTimer: ReturnType<typeof setTimeout> | null = null;

  function clearTaskSaveSuccessTimer() {
    if (taskSaveSuccessTimer !== null) {
      clearTimeout(taskSaveSuccessTimer);
      taskSaveSuccessTimer = null;
    }
  }

  function scheduleTaskSaveSuccessReset() {
    clearTaskSaveSuccessTimer();
    taskSaveSuccessTimer = setTimeout(() => {
      taskSaveSuccess.value = false;
      taskSaveSuccessTimer = null;
    }, 1800);
  }

  watch(
    defaultTaskCategoryId,
    (categoryId) => {
      if (!editingTaskId.value && !taskForm.value.categoryId && categoryId) {
        taskForm.value.categoryId = categoryId;
      }
    },
    { immediate: true },
  );

  onUnmounted(() => {
    clearTaskSaveSuccessTimer();
  });

  function resetTaskForm() {
    editingTaskId.value = null;
    taskForm.value = createEmptyTaskForm(defaultTaskCategoryId.value);
  }

  function openTaskModal() {
    taskSaveSuccess.value = false;
    resetTaskForm();
    isTaskModalOpen.value = true;
  }

  function closeTaskModal() {
    isTaskModalOpen.value = false;
    resetTaskForm();
  }

  function startEditTask(todo: TodoItem) {
    taskSaveSuccess.value = false;
    editingTaskId.value = todo.id;
    taskForm.value = {
      title: todo.title,
      categoryId: todo.categoryId,
      priority: todo.priority,
      description: todo.description || '',
    };
    onBeforeEditTask?.();
    isTaskModalOpen.value = true;
  }

  async function submitTask() {
    if (!taskForm.value.title.trim()) return;
    if (!taskForm.value.categoryId) return;

    const title = taskForm.value.title.trim();
    const description = taskForm.value.description.trim();
    const { priority, categoryId } = taskForm.value;

    let success = false;
    if (editingTaskId.value !== null) {
      const taskId = editingTaskId.value;
      success = await updateTodo(taskId, title, priority, categoryId, description);
    } else {
      success = await addTodo(title, priority, categoryId, description);
    }
    if (!success) return;

    taskSaveSuccess.value = true;
    scheduleTaskSaveSuccessReset();

    closeTaskModal();
  }

  return {
    isTaskModalOpen,
    editingTaskId,
    taskSaveSuccess,
    taskForm,
    openTaskModal,
    closeTaskModal,
    startEditTask,
    submitTask,
  };
}
