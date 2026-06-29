<script setup lang="ts">
import { computed, ref } from "vue";
import { NH2, NSpace, NSelect, NDatePicker, NButton, NText } from "naive-ui";
import { api, reportKinds, type ReportKind } from "../api";
import { useProjectsStore } from "../stores/projects";

const projects = useProjectsStore();
const kind = ref<ReportKind>("ResourceUtilization");
const dateRange = ref<[number, number]>([Date.parse("2026-06-29"), Date.parse("2026-07-12")]);
const fmt = ref<"csv" | "xlsx">("csv");
const projectId = ref<number | null>(null);
const msg = ref("");
const busy = ref(false);

const cn: Record<ReportKind, string> = {
  ResourceUtilization: "资源利用率",
  ProjectBurn: "项目预算消耗",
  AiDecisions: "AI 决策记录",
  Cost: "成本",
};

const kindOptions = reportKinds.map((k) => ({ label: cn[k], value: k }));
const fmtOptions = [
  { label: "CSV", value: "csv" },
  { label: "Excel", value: "xlsx" },
];
const projectOptions = computed(() => [
  { label: "全部项目", value: null },
  ...projects.items.map((p) => ({ label: p.name, value: p.id })),
]);

function fmtDate(ms: number): string {
  const d = new Date(ms);
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
}

async function doExport() {
  busy.value = true;
  msg.value = "";
  try {
    const start = fmtDate(dateRange.value[0]);
    const end = fmtDate(dateRange.value[1]);
    const ok = await api.exportReport(kind.value, projectId.value, start, end, fmt.value);
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
</script>

<template>
  <n-h2 style="margin-top: 0">报表 / Reports</n-h2>
  <n-space align="center" :size="8" wrap>
    <n-select v-model:value="kind" :options="kindOptions" style="width: 160px" />
    <n-select v-model:value="projectId" :options="projectOptions" placeholder="项目" style="width: 200px" />
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
    报表通过浏览器下载保存。PDF 导出需启用后端的 app/pdf feature。
  </n-text>
</template>
