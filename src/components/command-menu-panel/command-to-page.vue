<script setup lang="ts">
import type { Component } from "vue";
import { CommandGroup, CommandItem } from "@/components/ui/command";

import CommandItemHasIcon from "./command-item-has-icon.vue";

interface NavItem {
  title: string;
  url: string;
  icon?: Component;
}

const props = defineProps<{
  items: NavItem[];
}>();

const emit = defineEmits<{
  (e: "click"): void;
}>();

const router = useRouter();
const route = useRoute();

function commandItemClick(url: string) {
  emit("click");
  if (route.fullPath !== url) {
    router.push(url);
  }
}
</script>

<template>
  <CommandGroup heading="Pages">
    <CommandItem
      v-for="command in props.items"
      :key="command.title"
      :value="command.title"
      @click="commandItemClick(command.url)"
    >
      <CommandItemHasIcon :name="command.title" :icon="command.icon" />
    </CommandItem>
  </CommandGroup>
</template>
