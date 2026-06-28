<script setup lang="ts">
import ResourceForm from "../components/ResourceForm.vue";
import { useResourcesStore } from "../stores/resources";
import { onMounted } from "vue";
const resources = useResourcesStore();
onMounted(() => resources.load());
</script>
<template>
  <h2>资源 / Resources</h2>
  <ResourceForm />
  <ul>
    <li v-for="r in resources.items" :key="r.id">{{ r.name }} <span v-if="r.email">· {{ r.email }}</span></li>
  </ul>
</template>