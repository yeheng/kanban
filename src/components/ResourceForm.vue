<script setup lang="ts">
import { ref } from "vue";
import { NForm, NFormItem, NInput, NButton } from "naive-ui";
import { useResourcesStore } from "../stores/resources";
const resources = useResourcesStore();
const name = ref("");
const email = ref("");
async function submit() {
  if (!name.value.trim()) return;
  await resources.create(name.value, email.value || null);
  name.value = "";
  email.value = "";
}
</script>

<template>
  <n-form inline @submit.prevent="submit">
    <n-form-item label="姓名">
      <n-input v-model:value="name" placeholder="姓名" />
    </n-form-item>
    <n-form-item label="邮箱">
      <n-input v-model:value="email" placeholder="email (可选)" />
    </n-form-item>
    <n-form-item>
      <n-button type="primary" attr-type="submit">新建资源</n-button>
    </n-form-item>
  </n-form>
</template>
