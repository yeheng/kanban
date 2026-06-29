<script setup lang="ts">
import { computed, ref } from "vue";
import { NForm, NFormItem, NSelect, NDatePicker, NInput, NButton } from "naive-ui";
import { useCalendarStore } from "../stores/calendar";
import { useResourcesStore } from "../stores/resources";
const cal = useCalendarStore();
const resources = useResourcesStore();
const rid = ref<number | null>(null);
const day = ref<number | null>(null);
const frac = ref(1);
const reason = ref("");

const resourceOptions = computed(() =>
  resources.items.map((r) => ({ label: r.name, value: r.id })),
);
const fracOptions = [
  { label: "全天", value: 1 },
  { label: "半天", value: 0.5 },
];

function fmtDate(ms: number): string {
  const d = new Date(ms);
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
}

async function add() {
  if (rid.value == null || day.value == null) return;
  await cal.addTimeOff(rid.value, fmtDate(day.value), frac.value, reason.value || null);
  day.value = null;
  reason.value = "";
}
</script>

<template>
  <n-form inline>
    <n-form-item label="资源">
      <n-select v-model:value="rid" :options="resourceOptions" placeholder="选择资源" />
    </n-form-item>
    <n-form-item label="日期">
      <n-date-picker v-model:value="day" type="date" clearable />
    </n-form-item>
    <n-form-item label="类型">
      <n-select v-model:value="frac" :options="fracOptions" style="width: 100px" />
    </n-form-item>
    <n-form-item label="原因">
      <n-input v-model:value="reason" placeholder="请假原因" />
    </n-form-item>
    <n-form-item>
      <n-button type="primary" @click="add">添加请假</n-button>
    </n-form-item>
  </n-form>
</template>
