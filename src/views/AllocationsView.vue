<script setup lang="ts">
import { computed, h, onMounted, ref, watchEffect } from "vue";
import { NDataTable, NH2, NButton, NPopconfirm, NModal, NForm, NFormItem, NDatePicker, NInputNumber, NSpace } from "naive-ui";
import type { DataTableColumns } from "naive-ui";
import { useAllocationsStore } from "../stores/allocations";
import { useResourcesStore } from "../stores/resources";
import { useProjectsStore } from "../stores/projects";
import { useRefreshStore } from "../stores/refresh";
import AllocationForm from "../components/AllocationForm.vue";
import type { AllocationView } from "../types";

const allocations = useAllocationsStore();
const resources = useResourcesStore();
const projects = useProjectsStore();
const refreshBus = useRefreshStore();
onMounted(() => resources.load());
// Reading refreshBus.version.allocations inside the effect makes it a dependency, so a bump
// (e.g. after an AI accept) re-runs the load without a manual refresh (design G4).
watchEffect(async () => {
  void refreshBus.version.allocations;
  if (projects.current != null) await allocations.load(projects.current);
});

// Edit modal state
const editVisible = ref(false);
const editingId = ref<number | null>(null);
const editDateRange = ref<[number, number]>([0, 0]);
const editPercent = ref(0.5);

function fmtDate(ms: number): string {
  const d = new Date(ms);
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
}

function openEdit(row: AllocationView) {
  editingId.value = row.id;
  const start = Date.parse(row.start_date);
  const end = Date.parse(row.end_date);
  editDateRange.value = [start, end];
  editPercent.value = row.percent;
  editVisible.value = true;
}

async function saveEdit() {
  if (editingId.value == null || projects.current == null) return;
  await allocations.update(
    editingId.value,
    fmtDate(editDateRange.value[0]),
    fmtDate(editDateRange.value[1]),
    editPercent.value,
    projects.current,
  );
  editVisible.value = false;
}

const columns = computed<DataTableColumns<AllocationView>>(() => [
  { title: "资源", key: "resource_name" },
  { title: "任务", key: "task_title" },
  { title: "区间", key: "range", render: (row) => `${row.start_date} → ${row.end_date}` },
  { title: "投入", key: "percent", render: (row) => `${Math.round(row.percent * 100)}%` },
  { title: "来源", key: "source" },
  {
    title: "操作",
    key: "actions",
    width: 120,
    render: (row) => h(NSpace, { size: 4 }, () => [
      h(NButton, { size: "small", onClick: () => openEdit(row) }, { default: () => "编辑" }),
      h(NPopconfirm, { onPositiveClick: () => allocations.remove(row.id, projects.current!) }, {
        trigger: () => h(NButton, { size: "small", type: "error", quaternary: true }, { default: () => "删除" }),
        default: () => "确定删除此分配吗？",
      }),
    ]),
  },
]);
</script>

<template>
  <n-h2>分配 / Allocations</n-h2>
  <AllocationForm />
  <n-data-table :columns="columns" :data="allocations.items" :bordered="true" style="margin-top: 12px" />

  <n-modal v-model:show="editVisible" preset="card" title="编辑分配" style="width: 480px">
    <n-form>
      <n-form-item label="区间">
        <n-date-picker v-model:value="editDateRange" type="daterange" clearable />
      </n-form-item>
      <n-form-item label="投入比例">
        <n-input-number v-model:value="editPercent" :min="0.01" :max="1" :step="0.05" />
      </n-form-item>
      <n-space justify="end">
        <n-button @click="editVisible = false">取消</n-button>
        <n-button type="primary" @click="saveEdit">保存</n-button>
      </n-space>
    </n-form>
  </n-modal>
</template>
