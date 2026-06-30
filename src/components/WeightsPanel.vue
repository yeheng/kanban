<script setup lang="ts">
import { computed } from "vue";
import { NSpace, NSlider, NText, NInputNumber } from "naive-ui";
import { useOptimizationStore } from "../stores/optimization";
type WeightKey = "skill_fit" | "balance" | "budget";
const opt = useOptimizationStore();
const labels: WeightKey[] = ["skill_fit", "balance", "budget"];
const cn: Record<WeightKey, string> = { skill_fit: "技能最优", balance: "负载均衡", budget: "预算" };
</script>

<template>
  <div class="weights-panel">
    <n-space
      v-for="k in labels"
      :key="k"
      align="center"
      :size="8"
      style="margin: 4px 0"
    >
      <n-text style="width: 80px">{{ cn[k] }}</n-text>
      <n-slider
        :value="opt.weights[k]"
        :min="0"
        :max="1"
        :step="0.05"
        style="width: 160px"
        @update:value="(v: number) => { opt.weights[k] = v; opt.normalize(); }"
      />
      <n-text>{{ Math.round(opt.weights[k] * 100) }}%</n-text>
    </n-space>
    <n-text depth="3" class="weights-panel__note">
      权重已接入求解器：技能/负载权重影响候选排序与打分系数，预算权重达到主导时触发预算上限。权重随运行快照留存供复现。
    </n-text>
  </div>
</template>

<style scoped>
.weights-panel__note {
  display: block;
  font-size: 11px;
  margin-top: 6px;
  max-width: 320px;
}
</style>
