<script setup lang="ts">
import type { CategoryItem } from '@/data/todos';

const props = defineProps<{
  visible: boolean;
  categories: CategoryItem[];
  editingCategoryId: string | null;
  name: string;
  color: string;
  icon: string;
  presetColors: string[];
  presetIcons: string[];
}>();

const emit = defineEmits<{
  close: [];
  submit: [];
  'cancel-edit': [];
  'start-edit': [categoryId: string];
  'request-delete': [categoryId: string];
  'update:name': [value: string];
  'update:color': [value: string];
  'update:icon': [value: string];
}>();

function updateName(event: Event) {
  emit('update:name', (event.target as HTMLInputElement).value);
}
</script>

<template>
  <Transition name="fade-up">
    <div v-if="props.visible" class="modal-overlay" @click.self="emit('close')">
      <div class="modal-content">
        <header class="modal-header">
          <h2>管理分类</h2>
          <button class="close-btn" type="button" @click="emit('close')">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M18 6L6 18M6 6l12 12"></path>
            </svg>
          </button>
        </header>
        <div class="modal-body">
          <div class="category-list">
            <div v-for="cat in props.categories" :key="cat.id" class="category-row">
              <div class="cat-info">
                <span class="cat-icon-preview" :style="{ backgroundColor: cat.color + '20', color: cat.color }">
                  {{ cat.icon }}
                </span>
                <span>{{ cat.name }}</span>
              </div>
              <div class="category-actions">
                <button class="btn-icon-secondary" type="button" @click.stop.prevent="emit('start-edit', cat.id)" title="编辑分类">
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M12 20h9"></path>
                    <path d="M16.5 3.5a2.12 2.12 0 113 3L7 19l-4 1 1-4 12.5-12.5z"></path>
                  </svg>
                  <span>编辑</span>
                </button>
                <button class="btn-icon-danger" type="button" @click.stop.prevent="emit('request-delete', cat.id)" title="删除分类">
                  <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M3 6h18M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2M10 11v6M14 11v6"></path>
                  </svg>
                  <span>删除</span>
                </button>
              </div>
            </div>
            <div v-if="props.categories.length === 0" class="empty-text">无自定义分类</div>
          </div>

          <div class="add-category-box">
            <div class="category-form-header">
              <h3>{{ props.editingCategoryId ? '编辑分类' : '新建分类' }}</h3>
              <button v-if="props.editingCategoryId" class="btn-text" type="button" @click="emit('cancel-edit')">取消编辑</button>
            </div>
            <div class="form-row category-create-row">
              <div class="form-group emoji-picker-group">
                <label>图标</label>
                <div class="selected-icon-card" :style="{ backgroundColor: props.color + '20', color: props.color }">
                  <span class="selected-icon-emoji">{{ props.icon }}</span>
                  <span class="selected-icon-label">当前图标</span>
                </div>
              </div>
              <div class="form-group" style="flex: 1;">
                <label>名称</label>
                <input :value="props.name" type="text" placeholder="输入名称..." @input="updateName" @keyup.enter="emit('submit')" />
              </div>
            </div>
            <div class="form-group icon-picker-field">
              <label>选择图标</label>
              <div class="icon-picker-grid">
                <button
                  v-for="item in props.presetIcons"
                  :key="item"
                  type="button"
                  class="icon-choice"
                  :class="{ 'is-active': props.icon === item }"
                  :style="props.icon === item ? { backgroundColor: props.color + '20', color: props.color, borderColor: props.color + '55' } : {}"
                  @click="emit('update:icon', item)"
                >
                  <span>{{ item }}</span>
                </button>
              </div>
            </div>
            <div class="form-group mt-2">
              <label>主题色</label>
              <div class="color-picker" :style="{ color: props.color }">
                <button
                  v-for="item in props.presetColors"
                  :key="item"
                  type="button"
                  class="color-swatch"
                  :class="{ 'is-active': props.color === item }"
                  :style="{ backgroundColor: item }"
                  @click="emit('update:color', item)"
                >
                  <svg v-if="props.color === item" class="check-icon-color" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
                    <path d="M5 13l4 4L19 7" stroke-linecap="round" stroke-linejoin="round"/>
                  </svg>
                </button>
              </div>
            </div>
            <button class="btn-primary w-full mt-2" type="button" @click="emit('submit')" :disabled="!props.name.trim()">
              {{ props.editingCategoryId ? '保存修改' : '添加分类' }}
            </button>
            <button
              v-if="props.editingCategoryId"
              class="btn-danger-outline w-full"
              type="button"
              @click.stop.prevent="emit('request-delete', props.editingCategoryId!)"
            >
              删除当前分类
            </button>
          </div>
        </div>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.category-list {
  display: flex;
  flex-direction: column;
  max-height: 240px;
  overflow-y: auto;
  border-radius: var(--radius-md);
  border: 1px solid var(--c-border-light);
  background-color: var(--c-bg-app);
}

.category-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px;
  background-color: transparent;
  border-radius: 0;
  border: none;
  border-bottom: 1px solid var(--c-border-light);
  transition: background-color 0.2s ease;
}

.category-row:last-child {
  border-bottom: none;
}

.category-row:hover {
  background-color: var(--c-bg-surface);
}

.cat-info {
  display: flex;
  align-items: center;
  gap: 12px;
  font-weight: 500;
}

.cat-icon-preview {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-sm);
  font-size: 1.1rem;
}

.category-actions {
  display: flex;
  align-items: center;
  gap: 6px;
}

.btn-icon-secondary,
.btn-icon-danger {
  background: none;
  border: none;
  color: var(--c-text-muted);
  padding: 6px 10px;
  font-size: 0.82rem;
  font-weight: 600;
  cursor: pointer;
  border-radius: 6px;
  display: flex;
  align-items: center;
  gap: 4px;
  justify-content: center;
  transition: all 0.2s;
  opacity: 1;
}

.btn-icon-secondary:hover {
  color: var(--c-accent);
  background-color: rgba(10, 132, 255, 0.1);
  transform: scale(1.05);
}

.btn-icon-danger:hover {
  color: var(--c-danger);
  background-color: rgba(255, 59, 48, 0.1);
  transform: scale(1.05);
}

.empty-text {
  text-align: center;
  color: var(--c-text-muted);
  font-size: 0.9rem;
  padding: 20px 0;
}

.add-category-box {
  margin-top: 16px;
  padding-top: 16px;
  border-top: 1px dashed var(--c-border);
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.add-category-box h3 {
  font-size: 0.95rem;
  font-weight: 600;
  color: var(--c-text-secondary);
}

.category-form-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.btn-text {
  background: none;
  border: none;
  color: var(--c-accent);
  font-size: 0.85rem;
  font-weight: 600;
  cursor: pointer;
  padding: 0;
}

.btn-text:hover {
  text-decoration: underline;
}

.emoji-picker-group {
  flex: 0 0 84px !important;
}

.category-create-row {
  align-items: stretch;
}

.selected-icon-card {
  min-height: 88px;
  border-radius: 14px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 6px;
  border: 1px solid transparent;
  transition: var(--transition-fast);
}

.selected-icon-emoji {
  font-size: 1.8rem;
  line-height: 1;
}

.selected-icon-label {
  font-size: 0.72rem;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  opacity: 0.78;
}

.icon-picker-field {
  gap: 10px;
}

.icon-picker-grid {
  display: grid;
  grid-template-columns: repeat(6, minmax(0, 1fr));
  gap: 10px;
}

.icon-choice {
  height: 44px;
  border-radius: 12px;
  border: 1px solid var(--c-border-light);
  background-color: var(--c-bg-app);
  color: var(--c-text-primary);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 1.35rem;
  transition: transform 0.18s ease, border-color 0.18s ease, background-color 0.18s ease, box-shadow 0.18s ease;
}

.icon-choice:hover {
  transform: translateY(-1px);
  border-color: var(--c-border);
}

.icon-choice.is-active {
  box-shadow: 0 8px 18px rgba(10, 132, 255, 0.12);
}

.color-picker {
  display: flex;
  gap: 12px;
  padding: 4px 0;
}

.color-swatch {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  border: none;
  cursor: pointer;
  transition: transform 0.2s, opacity 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  color: white;
}

.color-swatch:hover {
  transform: scale(1.1);
  opacity: 0.9;
}

.color-swatch.is-active {
  transform: scale(1.1);
  box-shadow: 0 0 0 2px var(--c-bg-surface), 0 0 0 3px currentColor;
}

.check-icon-color {
  width: 14px;
  height: 14px;
  stroke: white;
}
</style>
