<script setup lang="ts">
import { ref } from "vue";
import { api, reportKinds, type ReportKind } from "../api";

const kind = ref<ReportKind>("ResourceUtilization");
const start = ref("2026-06-29");
const end = ref("2026-07-12");
const fmt = ref<"csv" | "xlsx">("csv");
const msg = ref("");
const busy = ref(false);
const cn: Record<ReportKind, string> = {
  ResourceUtilization: "资源利用率",
  ProjectBurn: "项目预算消耗",
  AiDecisions: "AI 决策记录",
  Cost: "成本",
};

async function doExport() {
  busy.value = true;
  msg.value = "";
  try {
    const ok = await api.exportReport(kind.value, null, start.value, end.value, fmt.value);
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
    const ok = await api.exportSnapshot(start.value, end.value);
    msg.value = ok ? "已导出快照 JSON" : "导出失败";
  } catch (e: unknown) {
    msg.value = e instanceof Error ? e.message : String(e);
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <h2 style="margin-top: 0">报表 / Reports</h2>
  <div style="margin-bottom: 8px">
    <select v-model="kind">
      <option v-for="k in reportKinds" :key="k" :value="k">{{ cn[k] }}</option>
    </select>
    窗口 <input v-model="start" type="date" /> – <input v-model="end" type="date" />
    格式
    <select v-model="fmt">
      <option value="csv">CSV</option>
      <option value="xlsx">Excel</option>
    </select>
  </div>
  <button :disabled="busy" @click="doExport">{{ busy ? "导出中…" : "导出报表" }}</button>
  <button :disabled="busy" @click="doSnapshot">导出人力快照 (JSON)</button>
  <p>{{ msg }}</p>
  <p style="color: #888; font-size: 12px">
    报表通过浏览器下载保存。PDF 导出需启用后端的 app/pdf feature。
  </p>
</template>
