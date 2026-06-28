<script setup lang="ts">
import { useOptimizationStore } from "../stores/optimization";
const opt = useOptimizationStore();
function pct(v: number) { return Math.round(v) + "%"; }
</script>
<template>
  <div v-if="opt.current">
    <h3>方案 #{{ opt.current.run_id }}</h3>
    <div>综合评分 <b>{{ pct(opt.current.plan.solution.metrics.overall) }}</b> · 技能 {{ pct(opt.current.plan.solution.metrics.skill_fit) }} · 排期覆盖 {{ pct(opt.current.plan.solution.metrics.utilization) }}</div>

    <h4>已分配 ({{ opt.current.plan.solution.assignments.length }})</h4>
    <table border="1" cellpadding="4" style="border-collapse:collapse">
      <tr><th>资源</th><th>任务</th><th>区间</th><th>投入</th><th>匹配分</th></tr>
      <tr v-for="a in opt.current.plan.solution.assignments" :key="a.task_id">
        <td>#{{ a.resource_id }}</td><td>#{{ a.task_id }}</td>
        <td>{{ a.start }} → {{ a.end }}</td><td>{{ Math.round(a.percent * 100) }}%</td>
        <td>{{ Math.round(a.score * 100) }}</td>
      </tr>
    </table>

    <p v-if="opt.current.plan.solution.unscheduled.length" style="color:#d03050">
      ⚠ 未排期任务：{{ opt.current.plan.solution.unscheduled.join(", ") }}
    </p>

    <h4>解释</h4>
    <pre style="white-space:pre-wrap;background:#f7f7fa;padding:8px;border-radius:4px">{{ opt.current.plan.explanation_md }}</pre>

    <div style="margin-top:8px">
      <button @click="opt.accept(opt.current!.run_id)">✓ 采纳（写入分配）</button>
      <button @click="opt.reject(opt.current!.run_id)">✗ 拒绝</button>
    </div>
  </div>
</template>
