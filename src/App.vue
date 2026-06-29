<script setup lang="ts">
import { zhCN, dateZhCN } from "naive-ui";
import { NConfigProvider } from "naive-ui";
import AppLayout from "./components/AppLayout.vue";
</script>

<template>
  <n-config-provider :locale="zhCN" :date-locale="dateZhCN">
    <AppLayout />
  </n-config-provider>
</template>
