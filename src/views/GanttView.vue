<script setup lang="ts">
import { ref, watchEffect } from "vue";
import { useGanttStore } from "../stores/gantt";
import { useProjectsStore } from "../stores/projects";
import { useResourcesStore } from "../stores/resources";
import GanttTimeline from "../components/GanttTimeline.vue";

const gantt = useGanttStore();
const projects = useProjectsStore();
const resources = useResourcesStore();
const err = ref<string | null>(null);
// default window: a wide span covering typical project activity
const start = ref("2026-06-29"); const end = ref("2026-08-09");

// In project mode, follow the sidebar's selected project.
watchEffect(async () => {
  if (gantt.mode === "project" && projects.current != null) {
    gantt.focusId = projects.current;
    await safeLoad();
  }
});
async function safeLoad() {
  try { err.value = null; await gantt.load(); } catch (e: unknown) { err.value = e instanceof Error ? e.message : String(e); }
}
async function onResource(id: number) { gantt.mode = "resource"; gantt.focusId = id; await safeLoad(); }
async function toProjectMode() { gantt.mode = "project"; await safeLoad(); }
</script>

<template>
  <h2 style="margin-top:0">Gantt</h2>
  <div style="margin-bottom:8px">
    模式：
    <button :disabled="gantt.mode==='project'" @click="toProjectMode">项目</button>
    <select v-model.number="projects.current" @change="safeLoad" :disabled="gantt.mode!=='project'">
      <option v-for="p in projects.items" :key="p.id" :value="p.id">{{ p.name }}</option>
    </select>
    <span> 或资源视角：</span>
    <select @change="onResource(+($event.target as HTMLSelectElement).value)">
      <option :value="0">—</option>
      <option v-for="r in resources.items" :key="r.id" :value="r.id">{{ r.name }}</option>
    </select>
    <span style="color:#d03050" v-if="err"> ⚠ {{ err }}（可能越出任务/资源时间窗）</span>
  </div>
  <GanttTimeline :start="start" :end="end" />
</template>
