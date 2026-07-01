<script setup lang="ts">
import { Slider } from "@/components/ui/slider";
import type { ObjectiveWeights } from "@/types";

const weights = defineModel<ObjectiveWeights>({ required: true });

type WeightKey = "skill_fit" | "balance" | "budget";
const labels: WeightKey[] = ["skill_fit", "balance", "budget"];
const cn: Record<WeightKey, string> = { skill_fit: "技能最优", balance: "负载均衡", budget: "预算" };

function updateWeight(key: WeightKey, value: number) {
  const next = { ...weights.value, [key]: value };
  const s = next.skill_fit + next.balance + next.budget;
  if (s > 0) {
    next.skill_fit /= s;
    next.balance /= s;
    next.budget /= s;
  }
  weights.value = next;
}
</script>

<template>
  <div class="weights-panel">
    <div
      v-for="k in labels"
      :key="k"
      class="flex items-center gap-2 my-1"
    >
      <span class="text-muted-foreground w-20 shrink-0">{{ cn[k] }}</span>
      <Slider
        :model-value="[weights[k]]"
        :min="0"
        :max="1"
        :step="0.05"
        class="w-40"
        @update:model-value="(v: number[] | undefined) => { if (v && v[0] !== undefined) updateWeight(k, v[0]); }"
      />
      <span class="text-muted-foreground w-12">{{ Math.round(weights[k] * 100) }}%</span>
    </div>
    <span class="text-muted-foreground text-xs mt-1.5 block max-w-xs">
      权重已接入求解器：技能/负载权重影响候选排序与打分系数，预算权重达到主导时触发预算上限。权重随运行快照留存供复现。
    </span>
  </div>
</template>
