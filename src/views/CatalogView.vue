<script setup lang="ts">
import { onMounted, ref } from "vue";
import { NH2, NH3, NSpace, NForm, NFormItem, NInput, NButton, NList, NListItem, NThing, NTag, NEmpty } from "naive-ui";
import { useCatalogStore } from "../stores/catalog";

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
  <n-h2 style="margin-top: 0">技能与标签 / Skills & Tags</n-h2>

  <n-h3>技能</n-h3>
  <n-form inline>
    <n-form-item label="名称">
      <n-input v-model:value="skillName" placeholder="技能名" @keyup.enter="addSkill" />
    </n-form-item>
    <n-form-item>
      <n-button type="primary" @click="addSkill">添加技能</n-button>
    </n-form-item>
  </n-form>
  <n-list v-if="catalog.skills.length" bordered>
    <n-list-item v-for="s in catalog.skills" :key="s.id">
      <n-thing :title="s.name" />
      <template #suffix>
        <n-tag size="small" :bordered="false">ID: {{ s.id }}</n-tag>
      </template>
    </n-list-item>
  </n-list>
  <n-empty v-else description="暂无技能" />

  <n-h3>标签</n-h3>
  <n-form inline>
    <n-form-item label="名称">
      <n-input v-model:value="tagName" placeholder="标签名" @keyup.enter="addTag" />
    </n-form-item>
    <n-form-item>
      <n-button type="primary" @click="addTag">添加标签</n-button>
    </n-form-item>
  </n-form>
  <n-space v-if="catalog.tags.length" :size="8">
    <n-tag v-for="t in catalog.tags" :key="t.id" :color="t.color ?? undefined">
      {{ t.name }}
    </n-tag>
  </n-space>
  <n-empty v-else description="暂无标签" />
</template>
