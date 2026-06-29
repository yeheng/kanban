<script setup lang="ts">
import { ref } from "vue";
import { NForm, NFormItem, NInput, NInputNumber, NButton, NSpace } from "naive-ui";
import { useProjectsStore } from "../stores/projects";
const projects = useProjectsStore();
const name = ref("");
const priority = ref(5);
const budget = ref(0);
async function submit() {
  if (!name.value.trim()) return;
  await projects.create(name.value, priority.value, budget.value);
  name.value = "";
}
</script>

<template>
  <n-form inline>
    <n-form-item label="项目名">
      <n-input v-model:value="name" placeholder="项目名" @keyup.enter="submit" />
    </n-form-item>
    <n-form-item label="优先级">
      <n-input-number v-model:value="priority" :min="1" :max="9" />
    </n-form-item>
    <n-form-item label="预算 PD">
      <n-input-number v-model:value="budget" :min="0" />
    </n-form-item>
    <n-form-item>
      <n-button type="primary" @click="submit">新建项目</n-button>
    </n-form-item>
  </n-form>
</template>
