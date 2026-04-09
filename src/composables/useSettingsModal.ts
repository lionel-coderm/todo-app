import { onUnmounted, ref, type Ref } from 'vue';
import {
  generateAiReport,
  getDefaultDataDir,
  type AppSettings,
  type AiApiMode,
  type AppTheme,
  type ReportPeriod,
} from '@/services/storageService';

interface SettingsFormState {
  theme: AppTheme;
  dataLocation: string;
  dataFormat: 'json' | 'sqlite';
  aiModel: string;
  aiBaseUrl: string;
  aiApiMode: AiApiMode;
  aiEndpoint: string;
  aiApiKey: string;
}

interface UpdateSettingsPayload {
  aiModel?: string;
  aiBaseUrl?: string;
  aiApiMode?: AiApiMode;
  aiEndpoint?: string;
  aiApiKey?: string;
}

interface UseSettingsModalOptions {
  currentSettings: Ref<AppSettings>;
  updateSettings: (
    newType: 'json' | 'sqlite',
    newDataDir?: string,
    newTheme?: AppTheme,
    aiConfig?: UpdateSettingsPayload,
  ) => Promise<void>;
}

const DEFAULT_DATA_DIR_FALLBACK_PLACEHOLDER = '系统默认目录（读取失败，留空仍使用默认目录）';

function createDefaultSettingsForm(): SettingsFormState {
  return {
    theme: 'light',
    dataLocation: '',
    dataFormat: 'json',
    aiModel: '',
    aiBaseUrl: '',
    aiApiMode: 'auto',
    aiEndpoint: '',
    aiApiKey: '',
  };
}

export function useSettingsModal(options: UseSettingsModalOptions) {
  const { currentSettings, updateSettings } = options;

  const isSettingsModalOpen = ref(false);
  const isSavingSettings = ref(false);
  const settingsSaveError = ref<string | null>(null);
  const settingsSaveSuccess = ref(false);
  const defaultDataDirPlaceholder = ref('加载中...');

  const isGeneratingReport = ref(false);
  const reportError = ref<string | null>(null);
  const reportContent = ref('');
  const reportPeriod = ref<ReportPeriod>('weekly');

  const settingsForm = ref<SettingsFormState>(createDefaultSettingsForm());

  let saveSuccessTimer: ReturnType<typeof setTimeout> | null = null;

  function clearSaveSuccessTimer() {
    if (saveSuccessTimer !== null) {
      clearTimeout(saveSuccessTimer);
      saveSuccessTimer = null;
    }
  }

  function scheduleSaveSuccessReset() {
    clearSaveSuccessTimer();
    saveSuccessTimer = setTimeout(() => {
      settingsSaveSuccess.value = false;
      saveSuccessTimer = null;
    }, 2000);
  }

  onUnmounted(() => {
    clearSaveSuccessTimer();
  });

  async function openSettingsModal() {
    settingsForm.value.dataFormat = currentSettings.value.storageType;
    settingsForm.value.theme = currentSettings.value.theme ?? 'light';
    settingsForm.value.dataLocation = currentSettings.value.dataDir ?? '';
    settingsForm.value.aiModel = currentSettings.value.aiModel ?? '';
    settingsForm.value.aiBaseUrl = currentSettings.value.aiBaseUrl ?? '';
    settingsForm.value.aiApiMode = currentSettings.value.aiApiMode ?? 'auto';
    settingsForm.value.aiEndpoint = currentSettings.value.aiEndpoint ?? '';
    settingsForm.value.aiApiKey = currentSettings.value.aiApiKey ?? '';

    settingsSaveError.value = null;
    settingsSaveSuccess.value = false;
    reportError.value = null;

    try {
      const dataDir = (await getDefaultDataDir()).trim();
      defaultDataDirPlaceholder.value = dataDir || DEFAULT_DATA_DIR_FALLBACK_PLACEHOLDER;
    } catch (err) {
      console.warn('[SettingsModal] 获取默认数据目录失败，使用中性占位文案:', err);
      defaultDataDirPlaceholder.value = DEFAULT_DATA_DIR_FALLBACK_PLACEHOLDER;
    }

    isSettingsModalOpen.value = true;
  }

  function closeSettingsModal() {
    isSettingsModalOpen.value = false;
  }

  async function handleSaveSettings() {
    if (isSavingSettings.value) return;

    settingsSaveError.value = null;
    settingsSaveSuccess.value = false;
    isSavingSettings.value = true;

    try {
      await updateSettings(
        settingsForm.value.dataFormat,
        settingsForm.value.dataLocation,
        settingsForm.value.theme,
        {
          aiModel: settingsForm.value.aiModel,
          aiBaseUrl: settingsForm.value.aiBaseUrl,
          aiApiMode: settingsForm.value.aiApiMode,
          aiEndpoint: settingsForm.value.aiEndpoint,
          aiApiKey: settingsForm.value.aiApiKey,
        },
      );
      settingsSaveSuccess.value = true;
      scheduleSaveSuccessReset();
    } catch (err) {
      settingsSaveError.value = String(err);
    } finally {
      isSavingSettings.value = false;
    }
  }

  async function handleGenerateReport(period: ReportPeriod) {
    if (isGeneratingReport.value) return;

    reportError.value = null;
    reportPeriod.value = period;
    isGeneratingReport.value = true;

    try {
      const generated = await generateAiReport(period);
      reportContent.value = generated;
    } catch (err) {
      reportError.value = String(err);
    } finally {
      isGeneratingReport.value = false;
    }
  }

  return {
    isSettingsModalOpen,
    isSavingSettings,
    settingsSaveError,
    settingsSaveSuccess,
    defaultDataDirPlaceholder,
    isGeneratingReport,
    reportError,
    reportContent,
    reportPeriod,
    settingsForm,
    openSettingsModal,
    closeSettingsModal,
    handleSaveSettings,
    handleGenerateReport,
  };
}
