<script setup lang="ts">
import { ref } from "vue";
import { useResourcesStore } from "../stores/resources";
const resources = useResourcesStore();
const name = ref(""); const email = ref("");
async function submit() { if (!name.value.trim()) return; await resources.create(name.value, email.value || null); name.value = ""; email.value = ""; }
</script>
<template>
  <form @submit.prevent="submit">
    <input v-model="name" placeholder="姓名" />
    <input v-model="email" placeholder="email (可选)" />
    <button>新建资源</button>
  </form>
</template>