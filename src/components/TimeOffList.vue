<script setup lang="ts">
import { ref } from "vue";
import { useCalendarStore } from "../stores/calendar";
import { useResourcesStore } from "../stores/resources";
const cal = useCalendarStore(); const resources = useResourcesStore();
const rid = ref<number | null>(null); const day = ref(""); const frac = ref(1); const reason = ref("");
async function add() { if (rid.value == null || !day.value) return; await cal.addTimeOff(rid.value, day.value, frac.value, reason.value || null); }
</script>
<template>
  <div>
    <select v-model.number="rid"><option :value="null">资源</option><option v-for="r in resources.items" :key="r.id" :value="r.id">{{ r.name }}</option></select>
    <input v-model="day" type="date" />
    <select v-model.number="frac"><option :value="1">全天</option><option :value="0.5">半天</option></select>
    <input v-model="reason" placeholder="原因" />
    <button @click="add">添加请假</button>
  </div>
</template>
