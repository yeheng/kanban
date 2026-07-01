<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import { toast } from "vue-sonner";
import { useSettingsStore } from "@/stores/settings";
import type { Settings } from "@/types";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  NumberField,
  NumberFieldContent,
  NumberFieldDecrement,
  NumberFieldIncrement,
  NumberFieldInput,
} from "@/components/ui/number-field";
import { Skeleton } from "@/components/ui/skeleton";
import { Switch } from "@/components/ui/switch";
import { Textarea } from "@/components/ui/textarea";
import { Loader2Icon } from "@lucide/vue";

const settings = useSettingsStore();
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
    toast.success("设置已保存");
  } catch (e) {
    toast.error(`保存失败: ${e instanceof Error ? e.message : String(e)}`);
  }
}

function reset() {
  if (settings.settings) {
    draft.value = { ...settings.settings };
  }
}

function updateNullableString(
  field: "ai_base_url" | "ai_api_key_enc" | "embed_base_url" | "embed_api_key_enc",
  value: string | number,
) {
  if (!draft.value) return;
  draft.value[field] = String(value || "");
}
</script>

<template>
  <div class="space-y-6">
    <div>
      <h1 class="text-2xl font-bold tracking-tight">设置</h1>
      <p class="text-muted-foreground">全局配置与偏好</p>
    </div>

    <div v-if="settings.loading || !draft" class="space-y-4">
      <Skeleton class="h-32 w-full" />
      <Skeleton class="h-32 w-full" />
      <Skeleton class="h-32 w-full" />
    </div>

    <div v-else class="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle>单位 / Units</CardTitle>
          <CardDescription>默认单位与换算系数</CardDescription>
        </CardHeader>
        <CardContent class="grid gap-6 sm:grid-cols-3">
          <div class="grid gap-2">
            <Label>默认单位</Label>
            <Select v-model="draft.default_unit">
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem v-for="o in unitOptions" :key="o.value" :value="o.value">
                  {{ o.label }}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div class="grid gap-2">
            <Label>每 PD 小时</Label>
            <NumberField v-model="draft.pd_hours" :step="0.5" :min="0.5">
              <NumberFieldContent>
                <NumberFieldDecrement />
                <NumberFieldInput />
                <NumberFieldIncrement />
              </NumberFieldContent>
            </NumberField>
          </div>
          <div class="grid gap-2">
            <Label>每 PM 人日</Label>
            <NumberField v-model="draft.pm_workdays" :step="1" :min="1">
              <NumberFieldContent>
                <NumberFieldDecrement />
                <NumberFieldInput />
                <NumberFieldIncrement />
              </NumberFieldContent>
            </NumberField>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>利用率阈值 / Thresholds</CardTitle>
          <CardDescription>过载、低载与颜色区间</CardDescription>
        </CardHeader>
        <CardContent class="grid gap-6 sm:grid-cols-4">
          <div class="grid gap-2">
            <Label>过载阈值</Label>
            <NumberField v-model="draft.overload_threshold" :step="0.05" :min="0">
              <NumberFieldContent>
                <NumberFieldDecrement />
                <NumberFieldInput />
                <NumberFieldIncrement />
              </NumberFieldContent>
            </NumberField>
          </div>
          <div class="grid gap-2">
            <Label>低载阈值</Label>
            <NumberField v-model="draft.underload_threshold" :step="0.05" :min="0">
              <NumberFieldContent>
                <NumberFieldDecrement />
                <NumberFieldInput />
                <NumberFieldIncrement />
              </NumberFieldContent>
            </NumberField>
          </div>
          <div class="grid gap-2">
            <Label>绿灯利用率</Label>
            <NumberField v-model="draft.utilization_green" :step="0.05" :min="0" :max="1">
              <NumberFieldContent>
                <NumberFieldDecrement />
                <NumberFieldInput />
                <NumberFieldIncrement />
              </NumberFieldContent>
            </NumberField>
          </div>
          <div class="grid gap-2">
            <Label>黄灯利用率</Label>
            <NumberField v-model="draft.utilization_yellow" :step="0.05" :min="0" :max="1">
              <NumberFieldContent>
                <NumberFieldDecrement />
                <NumberFieldInput />
                <NumberFieldIncrement />
              </NumberFieldContent>
            </NumberField>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Chat LLM</CardTitle>
          <CardDescription>对话模型与密钥配置</CardDescription>
        </CardHeader>
        <CardContent class="grid gap-6 sm:grid-cols-2 lg:grid-cols-3">
          <div class="grid gap-2">
            <Label>Provider</Label>
            <Select v-model="draft.ai_provider">
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem v-for="o in providerOptions" :key="o.value" :value="o.value">
                  {{ o.label }}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div class="grid gap-2">
            <Label>Base URL</Label>
            <Input
              :model-value="draft?.ai_base_url ?? ''"
              placeholder="可选，如 http://localhost:11434"
              @update:model-value="(v) => updateNullableString('ai_base_url', v)"
            />
          </div>
          <div class="grid gap-2">
            <Label>API Key</Label>
            <Input
              :model-value="draft?.ai_api_key_enc ?? ''"
              type="password"
              placeholder="可选"
              @update:model-value="(v) => updateNullableString('ai_api_key_enc', v)"
            />
          </div>
          <div class="grid gap-2">
            <Label>Chat Model</Label>
            <Input v-model="draft.ai_chat_model" />
          </div>
          <div class="grid gap-2">
            <Label>启用 LLM 解释</Label>
            <div class="flex h-9 items-center">
              <Switch v-model:checked="draft.use_llm_explainer" />
            </div>
          </div>
          <div class="grid gap-2">
            <Label>启用 LLM 建议</Label>
            <div class="flex h-9 items-center">
              <Switch v-model:checked="draft.use_llm_advisor" />
            </div>
            <p class="text-xs text-muted-foreground">
              开启后优化方案将附带结构化改进建议（换人/放宽窗口/解依赖等），可勾选后重跑求解器对比。
            </p>
          </div>
          <div class="grid gap-2">
            <Label>密钥存储</Label>
            <Select v-model="draft.secret_store">
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem v-for="o in secretStoreOptions" :key="o.value" :value="o.value">
                  {{ o.label }}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>
        </CardContent>
        <CardContent class="grid gap-6">
          <div class="grid gap-2">
            <Label>解释器系统提示词 (Preamble)</Label>
            <Textarea v-model="draft.ai_explanation_preamble" :rows="3" />
            <p class="text-muted-foreground text-xs">设定 LLM 的角色与全局要求。</p>
          </div>
          <div class="grid gap-2">
            <Label>解释器用户提示词模板 (Prompt Template)</Label>
            <Textarea v-model="draft.ai_explanation_prompt" :rows="12" />
            <p class="text-muted-foreground text-xs">
              可用占位符：{solver_backend}, {solver_status}, {weights_skill_fit}, {weights_balance},
              {weights_budget}, {budget_pd}, {resource_count}, {task_count}, {assignment_count},
              {unscheduled_count}, {resources}, {tasks}, {existing_allocs}, {assignments},
              {unscheduled}, {metrics_overall}, {metrics_skill_fit}, {metrics_scheduled_ratio},
              {metrics_fairness}, {full_context}
            </p>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Embedding LLM</CardTitle>
          <CardDescription>Embedding 模型与密钥配置</CardDescription>
        </CardHeader>
        <CardContent class="grid gap-6 sm:grid-cols-2 lg:grid-cols-3">
          <div class="grid gap-2">
            <Label>Provider</Label>
            <Select v-model="draft.embed_provider">
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem v-for="o in providerOptions" :key="o.value" :value="o.value">
                  {{ o.label }}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div class="grid gap-2">
            <Label>Base URL</Label>
            <Input
              :model-value="draft?.embed_base_url ?? ''"
              placeholder="可选，如 http://localhost:11434"
              @update:model-value="(v) => updateNullableString('embed_base_url', v)"
            />
          </div>
          <div class="grid gap-2">
            <Label>API Key</Label>
            <Input
              :model-value="draft?.embed_api_key_enc ?? ''"
              type="password"
              placeholder="可选"
              @update:model-value="(v) => updateNullableString('embed_api_key_enc', v)"
            />
          </div>
          <div class="grid gap-2">
            <Label>Embed Model</Label>
            <Input v-model="draft.embed_model" />
          </div>
          <div class="grid gap-2">
            <Label>启用语义匹配</Label>
            <div class="flex h-9 items-center">
              <Switch v-model:checked="draft.use_semantic_scorer" />
            </div>
          </div>
          <div class="grid gap-2">
            <Label>Embed 维度</Label>
            <NumberField v-model="draft.embed_dim" :step="1" :min="1">
              <NumberFieldContent>
                <NumberFieldDecrement />
                <NumberFieldInput />
                <NumberFieldIncrement />
              </NumberFieldContent>
            </NumberField>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>求解器 / Solver</CardTitle>
        </CardHeader>
        <CardContent class="grid gap-6 sm:grid-cols-2">
          <div class="grid gap-2">
            <Label>后端</Label>
            <Select v-model="draft.solver_backend">
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem v-for="o in solverOptions" :key="o.value" :value="o.value">
                  {{ o.label }}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div class="grid gap-2">
            <Label>超时 (ms)</Label>
            <NumberField v-model="draft.solver_timeout_ms" :step="100" :min="1">
              <NumberFieldContent>
                <NumberFieldDecrement />
                <NumberFieldInput />
                <NumberFieldIncrement />
              </NumberFieldContent>
            </NumberField>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>区域 / Locale</CardTitle>
        </CardHeader>
        <CardContent class="grid gap-6 sm:grid-cols-2">
          <div class="grid gap-2">
            <Label>Locale</Label>
            <Input v-model="draft.locale" />
          </div>
        </CardContent>
      </Card>

      <div class="flex items-center gap-4">
        <div class="flex gap-2">
          <Button type="button" :disabled="settings.saving || !draft" @click="save">
            <Loader2Icon v-if="settings.saving" class="mr-2 h-4 w-4 animate-spin" />
            {{ settings.saving ? "保存中..." : "保存设置" }}
          </Button>
          <Button type="button" variant="outline" :disabled="settings.saving || !draft" @click="reset">
            重置
          </Button>
        </div>
        <p v-if="settings.saving" class="text-muted-foreground text-sm">正在保存设置...</p>
      </div>
    </div>
  </div>
</template>
