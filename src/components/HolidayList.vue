<script setup lang="ts">
import { ref } from "vue";
import { useCalendarStore } from "../stores/calendar";
const cal = useCalendarStore();
const day = ref(""); const frac = ref(1); const name = ref("");
async function add() { if (!day.value) return; await cal.addHoliday(day.value, frac.value, name.value || null); day.value = ""; name.value = ""; }
</script>
<template>
  <div>
    <input v-model="day" type="date" />
    <select v-model.number="frac"><option :value="1">全天</option><option :value="0.5">半天</option></select>
    <input v-model="name" placeholder="名称" />
    <button @click="add">添加节假日</button>
    <ul><li v-for="h in cal.holidays" :key="h.id">{{ h.day }} · {{ h.fraction === 1 ? "全天" : "半天" }} · {{ h.name }}</li></ul>
  </div>
</template>
