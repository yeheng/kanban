<script setup lang="ts">
import { computed, h, onMounted, watchEffect } from "vue";
import { NDataTable, NH2, NButton, NPopconfirm } from "naive-ui";
import type { DataTableColumns } from "naive-ui";
import { useAllocationsStore } from "../stores/allocations";
import { useResourcesStore } from "../stores/resources";
import { useProjectsStore } from "../stores/projects";
import AllocationForm from "../components/AllocationForm.vue";
import type { AllocationView } from "../types";

const allocations = useAllocationsStore();
const resources = useResourcesStore();
const projects = useProjectsStore();
onMounted(() => resources.load());
watchEffect(async () => { if (projects.current != null) await allocations.load(projects.current); });

const columns = computed<DataTableColumns<AllocationView>>(() => [
  { title: "资源", key: "resource_name" },
  { title: "任务", key: "task_title" },
  { title: "区间", key: "range", render: (row) => `${row.start_date} → ${row.end_date}` },
  { title: "投入", key: "percent", render: (row) => `${Math.round(row.percent * 100)}%` },
  { title: "来源", key: "source" },
  {
    title: "操作",
    key: "actions",
    width: 80,
    render: (row) =>
      h(NPopconfirm, { onPositiveClick: () => allocations.remove(row.id, projects.current!) }, {
        trigger: () => h(NButton, { size: "small", type: "error", quaternary: true }, { default: () => "删除" }),
        default: () => "确定删除此分配吗？",
      }),
  },
]);
</script>

<template>
  <n-h2>分配 / Allocations</n-h2>
  <AllocationForm />
  <n-data-table :columns="columns" :data="allocations.items" :bordered="true" style="margin-top: 12px" />
</template>
