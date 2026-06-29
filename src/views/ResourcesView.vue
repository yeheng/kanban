<script setup lang="ts">
import { NH2, NList, NListItem, NThing, NText } from "naive-ui";
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
      <n-thing :title="r.name" :description="r.email ?? ''" />
    </n-list-item>
  </n-list>
</template>
