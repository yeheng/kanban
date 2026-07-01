<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import {
  NH2,
  NH3,
  NForm,
  NFormItem,
  NInput,
  NInputNumber,
  NSelect,
  NButton,
  NSpace,
  NSpin,
  NCard,
  useMessage,
} from "naive-ui";
import { useSettingsStore } from "../stores/settings";
import type { Settings } from "../types";

const settings = useSettingsStore();
const message = useMessage();

const draft = ref<Settings | null>(null);

watch(
  () => settings.settings,
  (s) => {
    if (s && !draft.value) {
      draft.value = { ...s };
    }
  },
  { immediate: true },
);

onMounted(async () => {
  await settings.load();
  if (settings.settings && !draft.value) {
    draft.value = { ...settings.settings };
  }
});

const unitOptions = [
  { label: "PD (人日)", value: "PD" },
  { label: "PM (人月)", value: "PM" },
];

const providerOptions = [
  { label: "Ollama", value: "ollama" },
  { label: "OpenAI", value: "openai" },
  { label: "Anthropic", value: "anthropic" },
  { label: "DeepSeek", value: "deepseek" },
];

const secretStoreOptions = [
  { label: "Keychain", value: "keychain" },
  { label: "Encrypted File", value: "encrypted_file" },
];

const solverOptions = [
  { label: "good_lp", value: "good_lp" },
  { label: "Greedy", value: "greedy" },
  { label: "Hungarian", value: "hungarian" },
];

async function save() {
  if (!draft.value) return;
  try {
    await settings.save(draft.value);
    message.success("设置已保存");
  } catch (e) {
    message.error(`保存失败: ${e instanceof Error ? e.message : String(e)}`);
  }
}

function reset() {
  if (settings.settings) {
    draft.value = { ...settings.settings };
  }
}
</script>

<template>
  <n-h2 style="margin-top: 0">设置 / Settings</n-h2>

  <n-spin v-if="settings.loading || !draft" description="加载中..." />

  <template v-else>
    <n-space vertical :size="16">
      <n-card title="单位 / Units">
        <n-form inline>
          <n-form-item label="默认单位">
            <n-select v-model:value="draft.default_unit" :options="unitOptions" style="width: 160px" />
          </n-form-item>
          <n-form-item label="每 PD 小时">
            <n-input-number v-model:value="draft.pd_hours" :step="0.5" :min="0.5" style="width: 140px" />
          </n-form-item>
          <n-form-item label="每 PM 人日">
            <n-input-number v-model:value="draft.pm_workdays" :step="1" :min="1" style="width: 140px" />
          </n-form-item>
        </n-form>
      </n-card>

      <n-card title="利用率阈值 / Thresholds">
        <n-form inline>
          <n-form-item label="过载阈值">
            <n-input-number v-model:value="draft.overload_threshold" :step="0.05" :min="0" style="width: 140px" />
          </n-form-item>
          <n-form-item label="低载阈值">
            <n-input-number v-model:value="draft.underload_threshold" :step="0.05" :min="0" style="width: 140px" />
          </n-form-item>
          <n-form-item label="绿灯利用率">
            <n-input-number v-model:value="draft.utilization_green" :step="0.05" :min="0" :max="1" style="width: 140px" />
          </n-form-item>
          <n-form-item label="黄灯利用率">
            <n-input-number v-model:value="draft.utilization_yellow" :step="0.05" :min="0" :max="1" style="width: 140px" />
          </n-form-item>
        </n-form>
      </n-card>

      <n-card title="AI / LLM">
        <n-form inline>
          <n-form-item label="Provider">
            <n-select v-model:value="draft.ai_provider" :options="providerOptions" style="width: 160px" />
          </n-form-item>
          <n-form-item label="Base URL">
            <n-input v-model:value="draft.ai_base_url" placeholder="可选，如 http://localhost:11434" style="width: 260px" />
          </n-form-item>
          <n-form-item label="API Key">
            <n-input v-model:value="draft.ai_api_key_enc" type="password" placeholder="可选" style="width: 220px" />
          </n-form-item>
          <n-form-item label="Chat Model">
            <n-input v-model:value="draft.ai_chat_model" style="width: 180px" />
          </n-form-item>
          <n-form-item label="Embed Model">
            <n-input v-model:value="draft.ai_embed_model" style="width: 180px" />
          </n-form-item>
          <n-form-item label="Embed 维度">
            <n-input-number v-model:value="draft.ai_embed_dim" :step="1" :min="1" style="width: 140px" />
          </n-form-item>
          <n-form-item label="密钥存储">
            <n-select v-model:value="draft.secret_store" :options="secretStoreOptions" style="width: 180px" />
          </n-form-item>
        </n-form>
      </n-card>

      <n-card title="求解器 / Solver">
        <n-form inline>
          <n-form-item label="后端">
            <n-select v-model:value="draft.solver_backend" :options="solverOptions" style="width: 160px" />
          </n-form-item>
          <n-form-item label="超时 (ms)">
            <n-input-number v-model:value="draft.solver_timeout_ms" :step="100" :min="1" style="width: 140px" />
          </n-form-item>
        </n-form>
      </n-card>

      <n-card title="区域 / Locale">
        <n-form inline>
          <n-form-item label="Locale">
            <n-input v-model:value="draft.locale" style="width: 140px" />
          </n-form-item>
        </n-form>
      </n-card>

      <n-space>
        <n-button type="primary" :loading="settings.saving" @click="save">保存设置</n-button>
        <n-button @click="reset">重置</n-button>
      </n-space>
    </n-space>
  </template>
</template>
