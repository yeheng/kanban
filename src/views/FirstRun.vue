<script setup lang="ts">
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
</script>

<template>
  <Card class="mx-auto mt-8 max-w-xl">
    <CardHeader>
      <CardTitle>首次启动</CardTitle>
    </CardHeader>
    <CardContent class="space-y-4">
      <p class="text-muted-foreground">
        设置数据库主口令（首次启用）—— 完整首启流程见 §6.8（后续任务）。
      </p>
      <p class="text-muted-foreground">
        当前开发模式：以 <code class="bg-muted rounded px-1">HRK_DB_PASSPHRASE</code> 环境变量启动加密库，否则使用未加密开发库。
      </p>
    </CardContent>
  </Card>
</template>
