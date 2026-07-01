<script setup lang="ts">
import { useListResourcesQuery } from "@/services/api/resources.api";
import WorkWeekEditor from "@/components/WorkWeekEditor.vue";
import HolidayList from "@/components/HolidayList.vue";
import TimeOffList from "@/components/TimeOffList.vue";

const resourcesQuery = useListResourcesQuery();
</script>

<template>
  <div class="space-y-6">
    <h1 class="text-2xl font-bold tracking-tight">日历 / Calendar</h1>
    <h2 class="text-xl font-semibold tracking-tight">工作周模板</h2>
    <WorkWeekEditor />
    <h2 class="text-xl font-semibold tracking-tight">节假日</h2>
    <HolidayList />
    <h2 class="text-xl font-semibold tracking-tight">请假 / 调休</h2>
    <TimeOffList :resources="resourcesQuery.data.value ?? []" />
  </div>
</template>
