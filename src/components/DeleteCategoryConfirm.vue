<script setup lang="ts">
const props = defineProps<{
  visible: boolean;
  categoryName: string;
  taskCount: number;
}>();

const emit = defineEmits<{
  close: [];
  confirm: [];
}>();
</script>

<template>
  <Transition name="fade-up">
    <div v-if="props.visible" class="modal-overlay" @click.self="emit('close')">
      <div class="modal-content confirm-content">
        <header class="modal-header">
          <h2>确认删除分类</h2>
          <button class="close-btn" type="button" @click="emit('close')">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M18 6L6 18M6 6l12 12"></path>
            </svg>
          </button>
        </header>
        <div class="modal-body confirm-body">
          <p class="confirm-text">
            确定删除分类「{{ props.categoryName }}」吗？
          </p>
          <p v-if="props.taskCount > 0" class="confirm-warning">
            该分类下 {{ props.taskCount }} 个任务也会被删除，且无法恢复。
          </p>
        </div>
        <footer class="modal-footer">
          <button class="btn-ghost" type="button" @click="emit('close')">取消</button>
          <button class="btn-danger-outline" type="button" @click="emit('confirm')">确认删除</button>
        </footer>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.confirm-content {
  max-width: 420px;
}

.confirm-body {
  gap: 10px;
}

.confirm-text {
  color: var(--c-text-primary);
  font-size: 0.95rem;
  line-height: 1.6;
}

.confirm-warning {
  color: var(--c-danger);
  font-size: 0.88rem;
  line-height: 1.5;
}
</style>
