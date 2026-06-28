<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useWorkloadStore } from "../stores/workload";
import { useResourcesStore } from "../stores/resources";
import { useProjectsStore } from "../stores/projects";
import { useTeamsStore } from "../stores/teams";
import UtilBar from "../components/UtilBar.vue";

const wl = useWorkloadStore();
const resources = useResourcesStore();
const projects = useProjectsStore();
const teams = useTeamsStore();
const start = ref("2026-06-29"); const end = ref("2026-07-03");
const selectedTeam = ref<number | null>(null);

async function refresh() {
  await resources.load();
  await wl.loadResourceSummaries(resources.items.map((r) => r.id), start.value, end.value);
  await wl.loadOverloads(start.value, end.value);
  if (projects.current != null) await wl.loadProjectBurn(projects.current);
  if (selectedTeam.value != null) await wl.loadTeamSummary(selectedTeam.value, start.value, end.value);
}
onMounted(async () => { await wl.loadThresholds(); await teams.load(); await refresh(); });
</script>
<template>
  <div>
    <h2 style="margin-top:0">Dashboard / 人力概览</h2>
    <div style="margin-bottom:12px">
      窗口 <input v-model="start" type="date" /> – <input v-model="end" type="date" />
      <button @click="refresh">刷新</button>
    </div>

    <h3>过载预警 ({{ wl.overloads.length }})</h3>
    <div v-for="o in wl.overloads" :key="o.resource_id" class="alert">
      ⚠ 资源 #{{ o.resource_id }} 利用率 {{ Math.round(o.utilization * 100) }}%
    </div>
    <p v-if="!wl.overloads.length">无过载 🎉</p>

    <h3>资源利用率</h3>
    <table>
      <tr v-for="s in wl.resourceSummaries" :key="s.resource_id">
        <td style="width:120px">资源 #{{ s.resource_id }}</td>
        <td><UtilBar :utilization="s.utilization" /></td>
        <td>{{ s.workload_pd.toFixed(1) }} / {{ s.capacity_pd.toFixed(1) }} PD</td>
      </tr>
    </table>

    <h3>项目健康（预算消耗）</h3>
    <div v-if="wl.projectBurn">
      {{ wl.projectBurn.allocated_pd.toFixed(1) }} / {{ wl.projectBurn.budget_pd.toFixed(1) }} PD
      ({{ Math.round(wl.projectBurn.usage * 100) }}%)
    </div>

    <h3>团队利用率</h3>
    <select v-model.number="selectedTeam" @change="refresh">
      <option :value="null">— 选择团队 —</option>
      <option v-for="t in teams.items" :key="t.id" :value="t.id">{{ t.name }}</option>
    </select>
    <div v-if="wl.teamSummary">
      <UtilBar :utilization="wl.teamSummary.utilization" />
      <small>过载成员：{{ wl.teamSummary.overloaded_members.join(", ") || "无" }}</small>
    </div>
  </div>
</template>
<style scoped>
.alert { background: #fff0f0; border: 1px solid #ffc0cb; padding: 4px 8px; border-radius: 4px; margin: 2px 0; }
table td { padding: 2px 6px; }
</style>
