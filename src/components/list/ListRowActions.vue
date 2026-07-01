<script setup lang="ts">
import { MoreHorizontalIcon } from "@lucide/vue";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";

defineEmits<{
  edit: [];
  delete: [];
}>();
</script>

<template>
  <DropdownMenu>
    <DropdownMenuTrigger as-child>
      <Button variant="ghost" size="icon" class="h-8 w-8">
        <MoreHorizontalIcon class="h-4 w-4" />
        <span class="sr-only">操作</span>
      </Button>
    </DropdownMenuTrigger>
    <DropdownMenuContent align="end">
      <DropdownMenuItem @click="$emit('edit')">编辑</DropdownMenuItem>
      <DropdownMenuItem class="text-destructive focus:text-destructive" @click="$emit('delete')">
        删除
      </DropdownMenuItem>
    </DropdownMenuContent>
  </DropdownMenu>
</template>
