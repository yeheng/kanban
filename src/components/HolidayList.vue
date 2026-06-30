<script setup lang="ts">
import { ref } from "vue";
import { NForm, NFormItem, NDatePicker, NSelect, NInput, NButton, NList, NListItem, NThing, NSpace, NPopconfirm } from "naive-ui";
import { useCalendarStore } from "../stores/calendar";
import { fmtDate } from "../utils/date";
const cal = useCalendarStore();
const day = ref<number | null>(null);
const frac = ref(1);
const name = ref("");

const fracOptions = [
  { label: "全天", value: 1 },
  { label: "半天", value: 0.5 },
];

async function add() {
  if (day.value == null) return;
  await cal.addHoliday(fmtDate(day.value), frac.value, name.value || null);
  day.value = null;
  name.value = "";
}
</script>

<template>
  <div>
    <n-form inline>
      <n-form-item label="日期">
        <n-date-picker v-model:value="day" type="date" clearable />
      </n-form-item>
      <n-form-item label="类型">
        <n-select v-model:value="frac" :options="fracOptions" style="width: 100px" />
      </n-form-item>
      <n-form-item label="名称">
        <n-input v-model:value="name" placeholder="节假日名称" />
      </n-form-item>
      <n-form-item>
        <n-button type="primary" @click="add">添加节假日</n-button>
      </n-form-item>
    </n-form>
    <n-list bordered>
      <n-list-item v-for="h in cal.holidays" :key="h.id">
        <n-thing :title="h.day" :description="`${h.fraction === 1 ? '全天' : '半天'} · ${h.name ?? ''}`" />
        <template #suffix>
          <n-popconfirm @positive-click="cal.removeHoliday(h.id)">
            <template #trigger>
              <n-button size="small" type="error" quaternary>删除</n-button>
            </template>
            确定删除此节假日吗？
          </n-popconfirm>
        </template>
      </n-list-item>
    </n-list>
  </div>
</template>
