<script setup lang="ts">
import { NH2, NList, NListItem, NThing, NText, NPopconfirm, NButton, NSpace, NTag } from "naive-ui";
import ResourceForm from "../components/ResourceForm.vue";
import { useResourcesStore } from "../stores/resources";
import { onMounted } from "vue";
const resources = useResourcesStore();
onMounted(() => resources.load());
</script>

<template>
  <n-h2>资源 / Resources</n-h2>
  <ResourceForm />
  <n-list bordered hoverable>
    <n-list-item v-for="r in resources.items" :key="r.id">
      <n-thing :title="r.name">
        <template v-if="r.email" #description>
          {{ r.email }}
        </template>
        <template #suffix>
          <n-popconfirm @positive-click="resources.remove(r.id)">
            <template #trigger>
              <n-button size="small" type="error" quaternary>删除</n-button>
            </template>
            确定删除资源 "{{ r.name }}" 吗？
          </n-popconfirm>
        </template>
      </n-thing>
    </n-list-item>
  </n-list>
</template>
