<script setup lang="ts">
import { NCard, NText, NCode } from "naive-ui";
</script>

<template>
  <n-card title="首次启动" style="max-width: 600px; margin: 2rem auto">
    <n-text>设置数据库主口令（首次启用）—— 完整首启流程见 §6.8（后续任务）。</n-text>
    <br /><br />
    <n-text>当前开发模式：以 <n-code code="HRK_DB_PASSPHRASE" /> 环境变量启动加密库，否则使用未加密开发库。</n-text>
  </n-card>
</template>
