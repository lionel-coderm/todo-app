import { ref, type Ref } from 'vue';
import type { CategoryItem, TodoItem } from '@/data/todos';

interface CategoryFormState {
  name: string;
  color: string;
  icon: string;
}

interface UseCategoryModalOptions {
  categories: Ref<CategoryItem[]>;
  todos: Ref<TodoItem[]>;
  addCategory: (name: string, color: string, icon: string) => Promise<boolean> | boolean;
  updateCategory: (id: string, name: string, color: string, icon: string) => Promise<boolean> | boolean;
  deleteCategory: (id: string) => Promise<boolean> | boolean;
}

const DEFAULT_CATEGORY_COLOR = '#0A84FF';
const DEFAULT_CATEGORY_ICON = '📁';

function createDefaultCategoryForm(): CategoryFormState {
  return {
    name: '',
    color: DEFAULT_CATEGORY_COLOR,
    icon: DEFAULT_CATEGORY_ICON,
  };
}

export function useCategoryModal(options: UseCategoryModalOptions) {
  const { categories, todos, addCategory, updateCategory, deleteCategory } = options;

  const isCategoryModalOpen = ref(false);
  const editingCategoryId = ref<string | null>(null);
  const categoryForm = ref<CategoryFormState>(createDefaultCategoryForm());

  const isDeleteCategoryConfirmOpen = ref(false);
  const pendingDeleteCategoryId = ref<string | null>(null);
  const pendingDeleteCategoryName = ref('');
  const pendingDeleteCategoryTaskCount = ref(0);

  const presetColors = ['#0A84FF', '#30D158', '#FF9F0A', '#FF453A', '#98989D', '#AF52DE'];
  const presetCategoryIcons = ['📁', '💼', '💻', '📚', '🏠', '🛒', '🏃', '💡', '🎯', '✈️', '🎵', '🍽️'];

  function resetCategoryForm() {
    editingCategoryId.value = null;
    categoryForm.value = createDefaultCategoryForm();
  }

  function openCategoryModal() {
    isCategoryModalOpen.value = true;
  }

  function closeCategoryModal() {
    isCategoryModalOpen.value = false;
    resetCategoryForm();
  }

  function startEditCategory(categoryId: string) {
    const target = categories.value.find((category) => category.id === categoryId);
    if (!target) return;
    editingCategoryId.value = categoryId;
    categoryForm.value = {
      name: target.name,
      color: target.color,
      icon: target.icon,
    };
  }

  function cancelEditCategory() {
    resetCategoryForm();
  }

  async function submitCategory() {
    if (!categoryForm.value.name.trim() || !categoryForm.value.icon.trim()) return;

    const name = categoryForm.value.name.trim();
    const icon = categoryForm.value.icon.trim();
    const { color } = categoryForm.value;

    let success = false;
    if (editingCategoryId.value) {
      success = await updateCategory(editingCategoryId.value, name, color, icon);
    } else {
      success = await addCategory(name, color, icon);
    }

    if (!success) return;
    resetCategoryForm();
  }

  function closeDeleteCategoryConfirm() {
    isDeleteCategoryConfirmOpen.value = false;
    pendingDeleteCategoryId.value = null;
    pendingDeleteCategoryName.value = '';
    pendingDeleteCategoryTaskCount.value = 0;
  }

  function handleDeleteCategory(categoryId: string) {
    const target = categories.value.find((category) => category.id === categoryId);
    if (!target) return;

    pendingDeleteCategoryId.value = categoryId;
    pendingDeleteCategoryName.value = target.name;
    pendingDeleteCategoryTaskCount.value = todos.value.filter((todo) => todo.categoryId === categoryId).length;
    isDeleteCategoryConfirmOpen.value = true;
  }

  async function confirmDeleteCategory() {
    const categoryId = pendingDeleteCategoryId.value;
    if (!categoryId) return;

    const success = await deleteCategory(categoryId);
    if (!success) return;

    if (editingCategoryId.value === categoryId) {
      resetCategoryForm();
    }

    closeDeleteCategoryConfirm();
  }

  return {
    isCategoryModalOpen,
    editingCategoryId,
    categoryForm,
    isDeleteCategoryConfirmOpen,
    pendingDeleteCategoryName,
    pendingDeleteCategoryTaskCount,
    presetColors,
    presetCategoryIcons,
    openCategoryModal,
    closeCategoryModal,
    startEditCategory,
    cancelEditCategory,
    submitCategory,
    handleDeleteCategory,
    closeDeleteCategoryConfirm,
    confirmDeleteCategory,
  };
}
