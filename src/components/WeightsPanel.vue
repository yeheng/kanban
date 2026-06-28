<script setup lang="ts">
import { useOptimizationStore } from "../stores/optimization";
type WeightKey = "skill_fit" | "balance" | "budget";
const opt = useOptimizationStore();
const labels: WeightKey[] = ["skill_fit", "balance", "budget"];
const cn: Record<WeightKey, string> = { skill_fit: "技能最优", balance: "负载均衡", budget: "预算" };
</script>
<template>
  <div class="weights">
    <div v-for="k in labels" :key="k">
      <small>{{ cn[k] }}</small>
      <input type="range" min="0" max="1" step="0.05" v-model.number="opt.weights[k]" @change="opt.normalize()" />
      {{ Math.round(opt.weights[k] * 100) }}
    </div>
    <p class="note">权重会记录到运行快照供复现；求解器尚按均衡目标求解，权重生效在后续迭代接入。</p>
  </div>
</template>
<style scoped>
.weights div { display: flex; align-items: center; gap: 8px; margin: 4px 0; font-size: 13px; }
small { width: 80px; }
.note { font-size: 11px; color: #888; margin: 6px 0 0; max-width: 320px; }
</style>
