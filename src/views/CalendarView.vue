<script setup lang="ts">
import { onMounted } from "vue";
import { useCalendarStore } from "../stores/calendar";
import { useResourcesStore } from "../stores/resources";
import WorkWeekEditor from "../components/WorkWeekEditor.vue";
import HolidayList from "../components/HolidayList.vue";
import TimeOffList from "../components/TimeOffList.vue";
const cal = useCalendarStore(); const resources = useResourcesStore();
onMounted(async () => { await resources.load(); await cal.loadHolidays(); await cal.loadWeek(); });
</script>
<template>
  <h2 style="margin-top:0">日历 / Calendar</h2>
  <h3>工作周模板</h3><WorkWeekEditor />
  <h3>节假日</h3><HolidayList />
  <h3>请假 / 调休</h3><TimeOffList />
</template>
