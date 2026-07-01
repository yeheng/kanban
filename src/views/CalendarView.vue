<script setup lang="ts">
import { onMounted } from "vue";
import { useCalendarStore } from "@/stores/calendar";
import { useResourcesStore } from "@/stores/resources";
import WorkWeekEditor from "@/components/WorkWeekEditor.vue";
import HolidayList from "@/components/HolidayList.vue";
import TimeOffList from "@/components/TimeOffList.vue";
const cal = useCalendarStore();
const resources = useResourcesStore();
onMounted(async () => { await resources.load(); await cal.loadHolidays(); await cal.loadWeek(); });
</script>

<template>
  <div class="space-y-6">
    <h1 class="text-2xl font-bold tracking-tight">日历 / Calendar</h1>
    <h2 class="text-xl font-semibold tracking-tight">工作周模板</h2>
    <WorkWeekEditor />
    <h2 class="text-xl font-semibold tracking-tight">节假日</h2>
    <HolidayList />
    <h2 class="text-xl font-semibold tracking-tight">请假 / 调休</h2>
    <TimeOffList />
  </div>
</template>
