<script setup lang="ts">
import { ref } from "vue";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  NumberField,
  NumberFieldContent,
  NumberFieldDecrement,
  NumberFieldIncrement,
  NumberFieldInput,
} from "@/components/ui/number-field";
import { useCreateProjectMutation } from "@/services/api/projects.api";

const createProject = useCreateProjectMutation();
const name = ref("");
const priority = ref(5);
const budget = ref(0);

async function submit() {
  if (!name.value.trim()) return;
  await createProject.mutateAsync({ name: name.value, priority: priority.value, budgetPd: budget.value });
  name.value = "";
  priority.value = 5;
  budget.value = 0;
}
</script>

<template>
  <div class="flex flex-wrap items-end gap-4">
    <div class="grid gap-2">
      <Label>项目名</Label>
      <Input v-model="name" placeholder="项目名" @keyup.enter="submit" />
    </div>
    <div class="grid gap-2">
      <Label>优先级</Label>
      <NumberField v-model="priority" :min="1" :max="9">
        <NumberFieldContent>
          <NumberFieldDecrement />
          <NumberFieldInput />
          <NumberFieldIncrement />
        </NumberFieldContent>
      </NumberField>
    </div>
    <div class="grid gap-2">
      <Label>预算 PD</Label>
      <NumberField v-model="budget" :min="0">
        <NumberFieldContent>
          <NumberFieldDecrement />
          <NumberFieldInput />
          <NumberFieldIncrement />
        </NumberFieldContent>
      </NumberField>
    </div>
    <Button @click="submit">新建项目</Button>
  </div>
</template>
