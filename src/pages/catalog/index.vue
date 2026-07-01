<script setup lang="ts">
import { onMounted, ref } from "vue";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent } from "@/components/ui/card";
import { useCatalogStore } from "@/stores/catalog";

const catalog = useCatalogStore();
const skillName = ref("");
const tagName = ref("");

onMounted(() => catalog.load());

async function addSkill() {
  if (!skillName.value.trim()) return;
  await catalog.ensureSkill(skillName.value);
  skillName.value = "";
}

async function addTag() {
  if (!tagName.value.trim()) return;
  await catalog.ensureTag(tagName.value, null);
  tagName.value = "";
}
</script>

<template>
  <h2 class="mt-0 text-2xl font-bold">技能与标签 / Skills & Tags</h2>

  <h3 class="text-xl font-semibold">技能</h3>
  <div class="flex flex-wrap items-end gap-4">
    <div class="grid gap-2">
      <Label for="skill-name">名称</Label>
      <Input id="skill-name" v-model="skillName" placeholder="技能名" @keyup.enter="addSkill" />
    </div>
    <Button @click="addSkill">添加技能</Button>
  </div>

  <Card v-if="catalog.skills.length" class="mt-4">
    <CardContent class="divide-y p-0">
      <div v-for="s in catalog.skills" :key="s.id" class="flex items-center justify-between px-4 py-3">
        <span class="font-medium">{{ s.name }}</span>
        <Badge variant="secondary">ID: {{ s.id }}</Badge>
      </div>
    </CardContent>
  </Card>
  <p v-else class="text-muted-foreground">暂无技能</p>

  <h3 class="text-xl font-semibold">标签</h3>
  <div class="flex flex-wrap items-end gap-4">
    <div class="grid gap-2">
      <Label for="tag-name">名称</Label>
      <Input id="tag-name" v-model="tagName" placeholder="标签名" @keyup.enter="addTag" />
    </div>
    <Button @click="addTag">添加标签</Button>
  </div>

  <div v-if="catalog.tags.length" class="mt-4 flex flex-wrap gap-2">
    <Badge v-for="t in catalog.tags" :key="t.id" :style="t.color ? { backgroundColor: t.color } : undefined">
      {{ t.name }}
    </Badge>
  </div>
  <p v-else class="text-muted-foreground">暂无标签</p>
</template>
