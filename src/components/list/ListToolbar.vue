<script setup lang="ts">
import { XIcon } from "@lucide/vue";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

const modelSearch = defineModel<string>("search", { default: "" });
const modelFilter = defineModel<string>("filter", { default: "all" });

withDefaults(
  defineProps<{
    searchPlaceholder?: string;
    filterLabel?: string;
    filterOptions?: { label: string; value: string }[];
    showReset?: boolean;
  }>(),
  { showReset: false },
);

const emit = defineEmits<{
  reset: [];
}>();
</script>

<template>
  <div class="flex items-center justify-between gap-4">
    <div class="flex flex-wrap items-center gap-2 flex-1">
      <Input
        v-model="modelSearch"
        :placeholder="searchPlaceholder ?? '搜索...'"
        class="h-8 w-[200px] lg:w-[280px]"
      />
      <Select v-if="filterOptions?.length" v-model="modelFilter">
        <SelectTrigger class="h-8 w-[150px]">
          <SelectValue :placeholder="filterLabel ?? '全部'" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="all">{{ filterLabel ?? "全部" }}</SelectItem>
          <SelectItem v-for="opt in filterOptions" :key="opt.value" :value="opt.value">
            {{ opt.label }}
          </SelectItem>
        </SelectContent>
      </Select>
      <Button
        v-if="showReset"
        variant="ghost"
        size="sm"
        class="h-8 px-2 lg:px-3"
        @click="emit('reset')"
      >
        重置
        <XIcon class="ml-2 h-4 w-4" />
      </Button>
    </div>
  </div>
</template>
