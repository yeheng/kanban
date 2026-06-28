<script setup lang="ts">
import { NConfigProvider } from "naive-ui";
import AppLayout from "./components/AppLayout.vue";
</script>

<template>
  <n-config-provider>
    <AppLayout />
  </n-config-provider>
</template>