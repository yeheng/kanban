<script setup lang="ts">
import { onMounted, watchEffect } from "vue";
import { useAllocationsStore } from "../stores/allocations";
import { useResourcesStore } from "../stores/resources";
import { useProjectsStore } from "../stores/projects";
import AllocationForm from "../components/AllocationForm.vue";

const allocations = useAllocationsStore();
const resources = useResourcesStore();
const projects = useProjectsStore();
onMounted(() => resources.load());
watchEffect(async () => { if (projects.current != null) await allocations.load(projects.current); });
</script>
<template>
  <h2 style="margin-top:0">分配 / Allocations</h2>
  <AllocationForm />
  <table border="1" cellpadding="4" style="border-collapse:collapse;margin-top:12px">
    <tr><th>资源</th><th>任务</th><th>区间</th><th>投入</th><th>来源</th></tr>
    <tr v-for="a in allocations.items" :key="a.id">
      <td>{{ a.resource_name }}</td><td>{{ a.task_title }}</td>
      <td>{{ a.start_date }} → {{ a.end_date }}</td>
      <td>{{ Math.round(a.percent * 100) }}%</td><td>{{ a.source }}</td>
    </tr>
  </table>
</template>
