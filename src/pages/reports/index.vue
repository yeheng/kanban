<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import DateRangePicker from "@/components/DateRangePicker.vue";
import { useGetReportCatalogQuery, reportKinds, exportReport, exportSnapshot, type ReportKind } from "@/services/api/reports.api";
import { useApiFetch } from "@/services/fetch";
import { useListProjectsQuery } from "@/services/api/projects.api";
import { fmtDate, parseDateStrict } from "@/utils/date";

const { apiFetch } = useApiFetch();
const projectsQuery = useListProjectsQuery();
const catalogQuery = useGetReportCatalogQuery();
const kind = ref<ReportKind>("ResourceUtilization");
const dateRange = ref<[number, number]>([parseDateStrict("2026-06-29"), parseDateStrict("2026-07-12")]);
const fmt = ref<string>("csv");
const projectId = ref<number | null>(null);
const msg = ref("");
const busy = ref(false);
const allProjectsValue = "__all__";

const cn: Record<ReportKind, string> = {
  ResourceUtilization: "资源利用率",
  TeamUtilization: "团队利用率",
  ProjectBurn: "项目预算消耗",
  AiDecisions: "AI 决策记录",
  Cost: "成本",
};

const kindOptions = reportKinds.map((k) => ({ label: cn[k], value: k }));
const projectOptions = computed(() => [
  { label: "全部项目", value: allProjectsValue },
  ...(projectsQuery.data.value ?? []).map((p) => ({ label: p.name, value: String(p.id) })),
]);
const projectValue = computed(() => projectId.value == null ? allProjectsValue : String(projectId.value));

const fmtOptions = computed(() => {
  const entry = catalogQuery.data.value?.find((e) => e.kind === kind.value);
  const formats = entry?.formats ?? ["csv", "xlsx"];
  return formats.map((f) => ({ label: f.toUpperCase(), value: f }));
});

const acceptsProject = computed(() => {
  const entry = catalogQuery.data.value?.find((e) => e.kind === kind.value);
  return entry?.accepts_project_id ?? false;
});

watch(kind, () => {
  const formats = fmtOptions.value.map((o) => o.value);
  if (!formats.includes(fmt.value)) fmt.value = formats[0] ?? "csv";
});

async function doExport() {
  busy.value = true;
  msg.value = "";
  try {
    const start = fmtDate(dateRange.value[0]);
    const end = fmtDate(dateRange.value[1]);
    const pid = acceptsProject.value ? projectId.value : null;
    const ok = await exportReport(apiFetch, kind.value, pid, start, end, fmt.value as "csv" | "xlsx" | "pdf");
    msg.value = ok ? `已导出 ${kind.value}.${fmt.value}` : "导出失败";
  } catch (e: unknown) {
    msg.value = e instanceof Error ? e.message : String(e);
  } finally {
    busy.value = false;
  }
}

async function doSnapshot() {
  busy.value = true;
  msg.value = "";
  try {
    const start = fmtDate(dateRange.value[0]);
    const end = fmtDate(dateRange.value[1]);
    const ok = await exportSnapshot(apiFetch, start, end);
    msg.value = ok ? "已导出快照 JSON" : "导出失败";
  } catch (e: unknown) {
    msg.value = e instanceof Error ? e.message : String(e);
  } finally {
    busy.value = false;
  }
}

function updateProject(value: unknown) {
  const s = String(value);
  projectId.value = s === allProjectsValue ? null : Number(s);
}
</script>

<template>
  <h2 class="text-2xl font-bold tracking-tight" style="margin-top: 0">报表 / Reports</h2>
  <div class="flex flex-wrap items-center gap-2">
    <Select v-model="kind">
      <SelectTrigger class="w-[160px]">
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        <SelectItem v-for="o in kindOptions" :key="o.value" :value="o.value">
          {{ o.label }}
        </SelectItem>
      </SelectContent>
    </Select>
    <Select
      v-if="acceptsProject"
      :model-value="projectValue"
      @update:model-value="updateProject"
    >
      <SelectTrigger class="w-[200px]">
        <SelectValue placeholder="项目" />
      </SelectTrigger>
      <SelectContent>
        <SelectItem v-for="o in projectOptions" :key="o.value" :value="o.value">
          {{ o.label }}
        </SelectItem>
      </SelectContent>
    </Select>
    <span>窗口</span>
    <DateRangePicker v-model="dateRange" />
    <span>格式</span>
    <Select v-model="fmt">
      <SelectTrigger class="w-[100px]">
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        <SelectItem v-for="o in fmtOptions" :key="o.value" :value="o.value">
          {{ o.label }}
        </SelectItem>
      </SelectContent>
    </Select>
  </div>
  <div class="mt-3 flex gap-2">
    <Button :disabled="busy" @click="doExport">导出报表</Button>
    <Button variant="outline" :disabled="busy" @click="doSnapshot">导出人力快照 (JSON)</Button>
  </div>
  <p v-if="msg" class="mt-2 text-sm text-muted-foreground">{{ msg }}</p>
  <p class="mt-2 text-xs text-muted-foreground">
    报表通过浏览器下载保存。可用格式由后端目录决定（PDF 需启用 app/pdf feature）。
  </p>
</template>
