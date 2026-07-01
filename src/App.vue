<script setup lang="ts">
import { VueQueryDevtools } from "@tanstack/vue-query-devtools";
import { Toaster } from "@/components/ui/sonner";

// 模板表达式作用域无法直接用 import.meta，故在 setup 中预计算。
// TanStack devtools v6 为组件（非插件），且内部按 NODE_ENV 自裁剪，
// 此处 isDev 守卫仅为显式表达"仅 dev"意图，与 tree-shaking 形成双重保险。
const isDev = import.meta.env.DEV;
</script>

<template>
  <router-view />
  <Toaster />
  <VueQueryDevtools v-if="isDev" />
</template>
