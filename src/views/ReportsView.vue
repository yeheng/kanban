<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { NH2, NSpace, NSelect, NDatePicker, NButton, NText } from "naive-ui";
import { api, reportKinds, type ReportKind, type ReportCatalogEntry } from "../api";
import { useProjectsStore } from "../stores/projects";
import { fmtDate, parseDateStrict } from "../utils/date";

const projects = useProjectsStore();
const kind = ref<ReportKind>("ResourceUtilization");
const dateRange = ref<[number, number]>([parseDateStrict("2026-06-29"), parseDateStrict("2026-07-12")]);
const fmt = ref<string>("csv");
const projectId = ref<number | null>(null);
const msg = ref("");
const busy = ref(false);
const allProjectsValue = "__all__";
const catalog = ref<ReportCatalogEntry[]>([]);

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
  ...projects.items.map((p) => ({ label: p.name, value: String(p.id) })),
]);
const projectValue = computed(() => projectId.value == null ? allProjectsValue : String(projectId.value));

// Available formats for the selected kind, driven by the backend catalog so unavailable
// formats (e.g. PDF without the app/pdf feature) are hidden instead of failing on export.
const fmtOptions = computed(() => {
  const entry = catalog.value.find((e) => e.kind === kind.value);
  const formats = entry?.formats ?? ["csv", "xlsx"];
  return formats.map((f) => ({ label: f.toUpperCase(), value: f }));
});
// Whether the selected kind accepts a project_id filter.
const acceptsProject = computed(() => {
  const entry = catalog.value.find((e) => e.kind === kind.value);
  return entry?.accepts_project_id ?? false;
});

onMounted(async () => {
  void projects.load();
  try { catalog.value = await api.getReportCatalog(); } catch { /* offline fallback: defaults */ }
});

// Keep the selected format valid when the kind changes.
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
    const ok = await api.exportReport(kind.value, pid, start, end, fmt.value as "csv" | "xlsx" | "pdf");
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
    const ok = await api.exportSnapshot(start, end);
    msg.value = ok ? "已导出快照 JSON" : "导出失败";
  } catch (e: unknown) {
    msg.value = e instanceof Error ? e.message : String(e);
  } finally {
    busy.value = false;
  }
}

function updateProject(value: string) {
  projectId.value = value === allProjectsValue ? null : Number(value);
}
</script>

<template>
  <n-h2 style="margin-top: 0">报表 / Reports</n-h2>
  <n-space align="center" :size="8" wrap>
    <n-select v-model:value="kind" :options="kindOptions" style="width: 160px" />
    <n-select
      v-if="acceptsProject"
      :value="projectValue"
      :options="projectOptions"
      placeholder="项目"
      @update:value="updateProject"
      style="width: 200px"
    />
    <span>窗口</span>
    <n-date-picker v-model:value="dateRange" type="daterange" clearable />
    <span>格式</span>
    <n-select v-model:value="fmt" :options="fmtOptions" style="width: 100px" />
  </n-space>
  <n-space style="margin-top: 12px" :size="8">
    <n-button type="primary" :loading="busy" @click="doExport">导出报表</n-button>
    <n-button :loading="busy" @click="doSnapshot">导出人力快照 (JSON)</n-button>
  </n-space>
  <n-text v-if="msg" style="margin-top: 8px">{{ msg }}</n-text>
  <n-text depth="3" style="font-size: 12px; margin-top: 8px">
    报表通过浏览器下载保存。可用格式由后端目录决定（PDF 需启用 app/pdf feature）。
  </n-text>
</template>
