<script setup lang="ts">
import { computed } from "vue";
import { RouterLink, useRoute } from "vue-router";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarRail,
} from "@/components/ui/sidebar";
import { useAppNav } from "@/composables/use-app-nav";
import { useProjectsStore } from "@/stores/projects";
import { useUnitStore } from "@/stores/unit";
import { useListProjectsQuery } from "@/services/api/projects.api";

const { items: navItems } = useAppNav();
const route = useRoute();
const activePath = computed(() => route.path);

const projects = useProjectsStore();
const unit = useUnitStore();
const projectsQuery = useListProjectsQuery();

const projectOptions = computed(() =>
  (projectsQuery.data.value ?? []).map((p) => ({ label: p.name, value: String(p.id) })),
);

function onProjectChange(value: unknown) {
  const id = Number(value);
  if (!Number.isNaN(id)) projects.select(id);
}
</script>

<template>
  <Sidebar collapsible="icon" class="z-50">
    <SidebarHeader>
      <SidebarMenu>
        <SidebarMenuItem>
          <SidebarMenuButton size="lg" as-child :tooltip="'Kanban'">
            <div class="flex items-center gap-2">
              <div class="flex aspect-square size-8 items-center justify-center rounded-lg bg-primary text-primary-foreground">
                <span class="text-sm font-bold">K</span>
              </div>
              <div class="grid flex-1 text-left text-sm leading-tight">
                <span class="truncate font-semibold">Kanban</span>
                <span class="truncate text-xs text-muted-foreground">资源调度</span>
              </div>
            </div>
          </SidebarMenuButton>
        </SidebarMenuItem>
      </SidebarMenu>
    </SidebarHeader>

    <SidebarContent>
      <SidebarGroup>
        <SidebarGroupLabel>导航</SidebarGroupLabel>
        <SidebarMenu>
          <SidebarMenuItem v-for="item in navItems" :key="item.url">
            <SidebarMenuButton
              as-child
              :tooltip="item.title"
              :is-active="activePath === item.url"
            >
              <RouterLink :to="item.url">
                <component :is="item.icon" class="size-4" />
                <span>{{ item.title }}</span>
              </RouterLink>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarGroup>
    </SidebarContent>

    <SidebarFooter>
      <div class="space-y-2 px-2 group-data-[collapsible=icon]:hidden">
        <div class="flex items-center gap-2">
          <span class="text-xs text-muted-foreground shrink-0">项目</span>
          <Select
            :model-value="projects.current ? String(projects.current) : undefined"
            @update:model-value="onProjectChange"
          >
            <SelectTrigger class="h-8 w-full">
              <SelectValue placeholder="选择项目" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem
                v-for="opt in projectOptions"
                :key="opt.value"
                :value="opt.value"
              >
                {{ opt.label }}
              </SelectItem>
            </SelectContent>
          </Select>
        </div>
        <div class="flex items-center gap-2">
          <span class="text-xs text-muted-foreground shrink-0">单位</span>
          <Select v-model="unit.unit">
            <SelectTrigger class="h-8 w-full">
              <SelectValue placeholder="单位" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="PD">PD</SelectItem>
              <SelectItem value="PM">PM</SelectItem>
            </SelectContent>
          </Select>
        </div>
      </div>
    </SidebarFooter>

    <SidebarRail />
  </Sidebar>
</template>
