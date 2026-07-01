<script setup lang="ts">
import { ref } from "vue";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useResourcesStore } from "@/stores/resources";

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
  <div class="flex flex-wrap items-end gap-4">
    <div class="grid gap-2">
      <Label for="resource-name">姓名</Label>
      <Input
        id="resource-name"
        v-model="name"
        placeholder="姓名"
        class="w-48"
        @keyup.enter="submit"
      />
    </div>
    <div class="grid gap-2">
      <Label for="resource-email">邮箱</Label>
      <Input
        id="resource-email"
        v-model="email"
        placeholder="email (可选)"
        class="w-64"
        @keyup.enter="submit"
      />
    </div>
    <div class="flex items-end">
      <Button @click="submit">新建资源</Button>
    </div>
  </div>
</template>
