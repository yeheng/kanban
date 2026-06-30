<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { NH2, NH3, NSpace, NDatePicker, NButton, NSelect, NAlert, NText, NStatistic, NTable } from "naive-ui";
import { useWorkloadStore } from "../stores/workload";
import { useResourcesStore } from "../stores/resources";
import { useProjectsStore } from "../stores/projects";
import { useTeamsStore } from "../stores/teams";
import { useUnitStore } from "../stores/unit";
import UtilBar from "../components/UtilBar.vue";

const wl = useWorkloadStore();
const resources = useResourcesStore();
const projects = useProjectsStore();
const teams = useTeamsStore();
const unit = useUnitStore();
const dateRange = ref<[number, number]>([Date.parse("2026-06-29"), Date.parse("2026-07-03")]);
const selectedTeam = ref<number | null>(null);

const teamOptions = computed(() => [
  { label: "— 选择团队 —", value: null },
  ...teams.items.map((t) => ({ label: t.name, value: t.id })),
]);

function fmtDate(ms: number): string {
  const d = new Date(ms);
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
}

async function refresh() {
  const start = fmtDate(dateRange.value[0]);
  const end = fmtDate(dateRange.value[1]);
  await resources.load();
  await wl.loadResourceSummaries(resources.items.map((r) => r.id), start, end);
  await wl.loadOverloads(start, end);
  if (projects.current != null) await wl.loadProjectBurn(projects.current);
  if (selectedTeam.value != null) await wl.loadTeamSummary(selectedTeam.value, start, end);
}
onMounted(async () => { await wl.loadThresholds(); await teams.load(); await refresh(); });
</script>

<template>
  <n-h2 style="margin-top: 0">Dashboard / 人力概览</n-h2>
  <n-space align="center" :size="8">
    <span>窗口</span>
    <n-date-picker v-model:value="dateRange" type="daterange" clearable />
    <n-button type="primary" @click="refresh">刷新</n-button>
  </n-space>

  <n-h3>过载预警 ({{ wl.overloads.length }})</n-h3>
  <n-space vertical>
    <n-alert v-for="o in wl.overloads" :key="o.resource_id" type="warning" show-icon>
      资源 #{{ o.resource_id }} 利用率 {{ Math.round(o.utilization * 100) }}%
    </n-alert>
    <n-text v-if="!wl.overloads.length" depth="3">无过载 🎉</n-text>
  </n-space>

  <n-h3>资源利用率</n-h3>
  <n-table :bordered="false" :single-line="false">
    <tr v-for="s in wl.resourceSummaries" :key="s.resource_id">
      <td style="width: 120px">资源 #{{ s.resource_id }}</td>
      <td><UtilBar :utilization="s.utilization" /></td>
      <td>{{ unit.formatPd(s.workload_pd) }} / {{ unit.formatPd(s.capacity_pd) }}</td>
    </tr>
  </n-table>

  <n-h3>项目健康（预算消耗）</n-h3>
  <n-statistic
    v-if="wl.projectBurn"
    label="预算消耗"
    :value="`${unit.formatPd(wl.projectBurn.allocated_pd)} / ${unit.formatPd(wl.projectBurn.budget_pd)} (${Math.round(wl.projectBurn.usage * 100)}%)`"
  />

  <n-h3>团队利用率</n-h3>
  <n-select v-model:value="selectedTeam" :options="teamOptions" @update:value="refresh" style="width: 200px" />
  <div v-if="wl.teamSummary" style="margin-top: 8px">
    <UtilBar :utilization="wl.teamSummary.utilization" />
    <n-text depth="3" style="font-size: 12px">过载成员：{{ wl.teamSummary.overloaded_members.join(", ") || "无" }}</n-text>
  </div>
</template>
