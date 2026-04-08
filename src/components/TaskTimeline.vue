<script setup lang="ts">
import { computed } from 'vue';
import { storeToRefs } from 'pinia';
import { useTodoStore } from '@/stores/todo';

const todoStore = useTodoStore();
const { visibleTodos } = storeToRefs(todoStore);

const timelineDates = computed(() => {
  return visibleTodos.value.map(todo => {
    return {
      id: todo.id,
      label: todo.dueLabel || '无日期',
      completed: todo.completed
    };
  });
});
</script>

<template>
  <div class="task-timeline">
    <div class="timeline-header">
      <h3>时间轴</h3>
    </div>
    
    <div class="timeline-list">
      <div 
        v-for="(item, index) in timelineDates" 
        :key="item.id"
        class="timeline-item"
      >
        <!-- 装饰线及圆点 -->
        <div class="timeline-dot-wrapper">
          <div class="timeline-dot" :class="item.completed ? 'completed' : 'created'"></div>
          <div v-if="index !== timelineDates.length - 1" class="timeline-line"></div>
        </div>
        
        <!-- 内容 -->
        <div class="timeline-content" style="display: flex; align-items: center;">
          <p class="activity-text" style="font-weight: 600; margin: 0; color: var(--c-text-muted);">{{ item.label }}</p>
        </div>
      </div>
      
      <div v-if="timelineDates.length === 0" style="color: var(--c-text-muted); font-size: 0.9rem; text-align: center; margin-top: 20px;">
        暂无日期信息
      </div>
    </div>
  </div>
</template>

<style scoped>
.task-timeline {
  padding: 12px 16px;
}

.timeline-header {
  margin-bottom: 24px;
}
.timeline-header h3 {
  font-size: 0.95rem;
  font-weight: 600;
  color: var(--c-text-primary);
  letter-spacing: 0.5px;
  margin: 0;
}

.timeline-list {
  display: flex;
  flex-direction: column;
}

.timeline-item {
  display: flex;
  gap: 20px;
  position: relative;
  min-height: 50px;
}

.timeline-dot-wrapper {
  position: relative;
  display: flex;
  flex-direction: column;
  align-items: center;
  width: 14px;
  padding-top: 4px;
}

.timeline-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background-color: var(--c-bg-surface);
  border: 2px solid var(--c-accent);
  z-index: 2;
  box-shadow: 0 0 0 3px var(--c-bg-app);
}

.timeline-dot.completed {
  border-color: var(--c-success);
  background-color: var(--c-success);
}

.timeline-line {
  position: absolute;
  top: 14px;
  bottom: -4px;
  left: 6px;
  width: 2px;
  background-color: var(--c-border-light);
}

.timeline-content {
  flex: 1;
  padding-bottom: 24px;
}

.timeline-item:last-child .timeline-content {
  padding-bottom: 0;
}

.activity-text {
  font-size: 0.9rem;
  font-weight: 600;
  color: var(--c-text-primary);
  margin: 0;
  font-variant-numeric: tabular-nums;
  letter-spacing: 0.5px;
}
</style>
