<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRoute, RouterLink } from "vue-router";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Separator } from "@/components/ui/separator";
import { Skeleton } from "@/components/ui/skeleton";
import {
  Sidebar,
  SidebarContent,
  SidebarGroup,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarInset,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarProvider,
  SidebarRail,
  SidebarTrigger,
} from "@/components/ui/sidebar";
import {
  LayoutDashboardIcon,
  KanbanIcon,
  FolderKanbanIcon,
  UsersIcon,
  TagsIcon,
  UsersRoundIcon,
  ListChecksIcon,
  CalendarIcon,
  BarChart3Icon,
  LayoutGridIcon,
  SparklesIcon,
  FileTextIcon,
  SettingsIcon,
} from "@lucide/vue";
import { useProjectsStore } from "@/stores/projects";
import { useCatalogStore } from "@/stores/catalog";
import { useUnitStore } from "@/stores/unit";

const projects = useProjectsStore();
const catalog = useCatalogStore();
const unit = useUnitStore();
const ready = ref(false);
const route = useRoute();

const navItems = [
  { to: "/dashboard", label: "仪表盘 Dashboard", icon: LayoutDashboardIcon },
  { to: "/kanban", label: "看板 Kanban", icon: KanbanIcon },
  { to: "/projects", label: "项目 Projects", icon: FolderKanbanIcon },
  { to: "/resources", label: "资源 Resources", icon: UsersIcon },
  { to: "/teams", label: "团队 Teams", icon: UsersRoundIcon },
  { to: "/allocations", label: "分配 Allocations", icon: ListChecksIcon },
  { to: "/calendar", label: "日历 Calendar", icon: CalendarIcon },
  { to: "/gantt", label: "甘特图 Gantt", icon: BarChart3Icon },
  { to: "/calendar-grid", label: "占用网格 Calendar Grid", icon: LayoutGridIcon },
  { to: "/catalog", label: "技能标签 Catalog", icon: TagsIcon },
  { to: "/ai", label: "AI 优化 Optimization", icon: SparklesIcon },
  { to: "/reports", label: "报表 Reports", icon: FileTextIcon },
  { to: "/settings", label: "设置 Settings", icon: SettingsIcon },
];

const activePath = computed(() => route.path);

const projectOptions = computed(() =>
  projects.items.map((p) => ({ label: p.name, value: String(p.id) })),
);

function onProjectChange(value: unknown) {
  const id = Number(value);
  if (!Number.isNaN(id)) projects.select(id);
}

onMounted(async () => {
  for (let i = 0; i < 40; i++) {
    try {
      await projects.load();
      await catalog.load();
      await unit.loadGlobal();
      ready.value = true;
      return;
    } catch {
      await new Promise((r) => setTimeout(r, 100));
    }
  }
});
</script>

<template>
  <SidebarProvider>
    <Sidebar collapsible="icon" class="z-50">
      <SidebarHeader>
        <div class="font-semibold text-lg px-2 truncate">Kanban</div>
      </SidebarHeader>

      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupLabel>导航</SidebarGroupLabel>
          <SidebarMenu>
            <SidebarMenuItem v-for="item in navItems" :key="item.to">
              <SidebarMenuButton
                as-child
                :tooltip="item.label"
                :is-active="activePath === item.to"
              >
                <RouterLink :to="item.to">
                  <component :is="item.icon" class="size-4" />
                  <span>{{ item.label }}</span>
                </RouterLink>
              </SidebarMenuButton>
            </SidebarMenuItem>
          </SidebarMenu>
        </SidebarGroup>
      </SidebarContent>

      <SidebarRail />
    </Sidebar>

    <SidebarInset>
      <header class="flex h-14 items-center gap-3 border-b bg-card px-4 shrink-0">
        <SidebarTrigger />
        <Separator orientation="vertical" class="h-6" />

        <div class="flex-1" />

        <div class="flex items-center gap-3">
          <div class="flex items-center gap-2">
            <span class="text-xs text-muted-foreground hidden md:inline">项目</span>
            <Select
              :model-value="projects.current ? String(projects.current) : undefined"
              @update:model-value="onProjectChange"
            >
              <SelectTrigger class="w-[140px] md:w-[180px]">
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

          <Separator orientation="vertical" class="h-6 hidden sm:block" />

          <div class="flex items-center gap-2">
            <span class="text-xs text-muted-foreground hidden md:inline">单位</span>
            <Select v-model="unit.unit">
              <SelectTrigger class="w-[80px] md:w-[100px]">
                <SelectValue placeholder="单位" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="PD">PD</SelectItem>
                <SelectItem value="PM">PM</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>
      </header>

      <main class="flex-1 overflow-auto p-6">
        <div v-if="!ready" class="space-y-4">
          <Skeleton class="h-8 w-48" />
          <Skeleton class="h-32 w-full" />
          <Skeleton class="h-32 w-full" />
        </div>
        <router-view v-else />
      </main>
    </SidebarInset>
  </SidebarProvider>
</template>
