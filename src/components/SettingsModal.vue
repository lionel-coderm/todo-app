<script setup lang="ts">
const props = defineProps<{
  visible: boolean;
  theme: string;
  dataLocation: string;
  dataFormat: 'json' | 'sqlite';
  aiModel: string;
  aiBaseUrl: string;
  aiApiKey: string;
  defaultDataDirPlaceholder: string;
  isSaving: boolean;
  error: string | null;
  success: boolean;
}>();

const emit = defineEmits<{
  close: [];
  save: [];
  'update:theme': [value: string];
  'update:dataLocation': [value: string];
  'update:dataFormat': [value: 'json' | 'sqlite'];
  'update:aiModel': [value: string];
  'update:aiBaseUrl': [value: string];
  'update:aiApiKey': [value: string];
}>();

function updateInput(name: 'theme' | 'dataLocation' | 'aiModel' | 'aiBaseUrl' | 'aiApiKey', event: Event) {
  const value = (event.target as HTMLInputElement).value;
  if (name === 'theme') emit('update:theme', value);
  if (name === 'dataLocation') emit('update:dataLocation', value);
  if (name === 'aiModel') emit('update:aiModel', value);
  if (name === 'aiBaseUrl') emit('update:aiBaseUrl', value);
  if (name === 'aiApiKey') emit('update:aiApiKey', value);
}

function updateDataFormat(event: Event) {
  emit('update:dataFormat', (event.target as HTMLInputElement).value as 'json' | 'sqlite');
}
</script>

<template>
  <Transition name="fade-up">
    <div v-if="props.visible" class="modal-overlay" @click.self="emit('close')">
      <div class="modal-content settings-modal-content">
        <header class="modal-header">
          <h2>系统设置</h2>
          <button class="close-btn" type="button" @click="emit('close')">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M18 6L6 18M6 6l12 12"></path>
            </svg>
          </button>
        </header>
        <div class="modal-body settings-body">
          <div class="settings-section">
            <h3 class="settings-section-title">外观体验</h3>
            <div class="settings-card">
              <div class="form-group">
                <label>主题模式</label>
                <div class="settings-radio-group">
                  <label class="settings-radio-label">
                    <input
                      type="radio"
                      :checked="props.theme === 'light'"
                      value="light"
                      @change="updateInput('theme', $event)"
                    />
                    亮色模式
                  </label>
                  <label class="settings-radio-label">
                    <input
                      type="radio"
                      :checked="props.theme === 'dark'"
                      value="dark"
                      @change="updateInput('theme', $event)"
                    />
                    深色模式
                  </label>
                </div>
              </div>
            </div>
          </div>

          <div class="settings-section">
            <h3 class="settings-section-title">数据管理</h3>
            <div class="settings-card settings-stack">
              <div class="form-group">
                <label>数据保存位置</label>
                <input
                  type="text"
                  :value="props.dataLocation"
                  :placeholder="props.defaultDataDirPlaceholder"
                  class="settings-input"
                  @input="updateInput('dataLocation', $event)"
                />
                <span class="settings-tip">留空则使用上方默认目录，支持 ~ 路径</span>
              </div>
              <div class="form-group">
                <label>数据保存格式</label>
                <div class="settings-radio-group">
                  <label class="settings-radio-label">
                    <input
                      type="radio"
                      :checked="props.dataFormat === 'json'"
                      value="json"
                      @change="updateDataFormat"
                    />
                    以文件形式 (JSON)
                  </label>
                  <label class="settings-radio-label">
                    <input
                      type="radio"
                      :checked="props.dataFormat === 'sqlite'"
                      value="sqlite"
                      @change="updateDataFormat"
                    />
                    使用 SQLite
                  </label>
                </div>
              </div>
            </div>
          </div>

          <div class="settings-section">
            <h3 class="settings-section-title">AI 模型配置 (职场助手)</h3>
            <div class="settings-card settings-stack">
              <p class="settings-tip settings-desc">
                配置大语言模型后，可根据您完成的待办事项，一键智能生成结构化的周报、月报。
              </p>
              <div class="form-group">
                <label>API Key</label>
                <input
                  type="password"
                  :value="props.aiApiKey"
                  placeholder="点击输入专属授权 API 密钥..."
                  class="settings-input"
                  @input="updateInput('aiApiKey', $event)"
                />
              </div>
              <div class="form-group">
                <label>模型名称</label>
                <input
                  type="text"
                  :value="props.aiModel"
                  placeholder="例如: gpt-4o, gemini-1.5-pro"
                  class="settings-input"
                  @input="updateInput('aiModel', $event)"
                />
              </div>
              <div class="form-group">
                <label>请求地址</label>
                <input
                  type="text"
                  :value="props.aiBaseUrl"
                  placeholder="例如: https://api.openai.com/v1"
                  class="settings-input"
                  @input="updateInput('aiBaseUrl', $event)"
                />
              </div>
            </div>
          </div>

          <div class="settings-section">
            <h3 class="settings-section-title">关于作者</h3>
            <div class="settings-card author-card">
              <div class="author-avatar">L</div>
              <div>
                <h4 class="author-name">Lionel</h4>
                <p class="author-desc">
                  热爱编码与生活，追求极致的用户体验与设计，致力于打造好用的效率工具。
                </p>
                <a href="https://github.com/lionel-coderm" target="_blank" class="author-link">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22"></path>
                  </svg>
                  @lionel GitHub
                </a>
              </div>
            </div>
          </div>
        </div>
        <footer class="modal-footer settings-footer">
          <div v-if="props.error" class="settings-alert settings-alert-error">⚠️ {{ props.error }}</div>
          <div v-if="props.success" class="settings-alert settings-alert-success">✅ 设置已保存</div>
          <div class="settings-footer-actions">
            <button class="btn-ghost settings-btn-close" type="button" @click="emit('close')">关闭</button>
            <button class="btn-primary settings-btn-save" type="button" @click="emit('save')" :disabled="props.isSaving">
              <span v-if="props.isSaving">保存中...</span>
              <span v-else>保存设置</span>
            </button>
          </div>
        </footer>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.settings-modal-content {
  max-width: 480px;
}

.settings-body {
  gap: 24px;
  max-height: 65vh;
  overflow-y: auto;
}

.settings-section-title {
  font-size: 0.85rem;
  margin-bottom: 12px;
  color: var(--c-text-secondary);
  font-weight: 600;
  text-transform: uppercase;
}

.settings-card {
  background-color: var(--c-bg-app);
  border: 1px solid var(--c-border-light);
  border-radius: 8px;
  padding: 16px;
}

.settings-stack {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.settings-radio-group {
  display: flex;
  gap: 20px;
  margin-top: 8px;
}

.settings-radio-label {
  display: flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
  font-size: 0.95rem;
  color: var(--c-text-primary);
}

.settings-radio-label input {
  accent-color: var(--c-accent);
}

.settings-input {
  margin-top: 8px;
  width: 100%;
  box-sizing: border-box;
}

.settings-tip {
  font-size: 0.78rem;
  color: var(--c-text-muted);
  margin-top: 2px;
}

.settings-desc {
  margin: 0;
  line-height: 1.5;
}

.author-card {
  display: flex;
  align-items: flex-start;
  gap: 16px;
}

.author-avatar {
  width: 52px;
  height: 52px;
  border-radius: 50%;
  background-color: var(--c-accent-bg);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--c-accent);
  font-size: 1.6rem;
  font-weight: bold;
  flex-shrink: 0;
}

.author-name {
  font-size: 1.05rem;
  color: var(--c-text-primary);
  margin: 0 0 6px 0;
}

.author-desc {
  color: var(--c-text-muted);
  font-size: 0.85rem;
  margin: 0;
  line-height: 1.5;
}

.author-link {
  color: var(--c-accent);
  font-size: 0.85rem;
  text-decoration: none;
  margin-top: 8px;
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.settings-footer {
  flex-direction: column;
  gap: 10px;
}

.settings-alert {
  width: 100%;
  padding: 10px 14px;
  border-radius: 8px;
  font-size: 0.85rem;
  line-height: 1.5;
}

.settings-alert-error {
  background: rgba(255, 59, 48, 0.1);
  border: 1px solid rgba(255, 59, 48, 0.3);
  color: var(--c-danger);
}

.settings-alert-success {
  background: rgba(48, 209, 88, 0.1);
  border: 1px solid rgba(48, 209, 88, 0.3);
  color: var(--c-success);
}

.settings-footer-actions {
  display: flex;
  gap: 12px;
  width: 100%;
}

.settings-btn-close {
  flex: 1;
}

.settings-btn-save {
  flex: 2;
}
</style>
