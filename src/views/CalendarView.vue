<script setup lang="ts">
import { onMounted } from "vue";
import { NH2, NH3, NSpace } from "naive-ui";
import { useCalendarStore } from "../stores/calendar";
import { useResourcesStore } from "../stores/resources";
import WorkWeekEditor from "../components/WorkWeekEditor.vue";
import HolidayList from "../components/HolidayList.vue";
import TimeOffList from "../components/TimeOffList.vue";
const cal = useCalendarStore();
const resources = useResourcesStore();
onMounted(async () => { await resources.load(); await cal.loadHolidays(); await cal.loadWeek(); });
</script>

<template>
  <n-h2 style="margin-top: 0">日历 / Calendar</n-h2>
  <n-h3>工作周模板</n-h3>
  <WorkWeekEditor />
  <n-h3>节假日</n-h3>
  <HolidayList />
  <n-h3>请假 / 调休</n-h3>
  <TimeOffList />
</template>
