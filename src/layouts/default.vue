<script setup lang="ts">
import { computed, watch } from "vue";
import { Separator } from "@/components/ui/separator";
import { Skeleton } from "@/components/ui/skeleton";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { SidebarInset, SidebarProvider, SidebarTrigger } from "@/components/ui/sidebar";
import AppSidebar from "@/components/app-sidebar/index.vue";
import ToggleTheme from "@/components/toggle-theme.vue";
import ThemePopover from "@/components/custom-theme/theme-popover.vue";
import CommandMenuPanel from "@/components/command-menu-panel/index.vue";
import { useThemeStore } from "@/stores/theme";
import { useProjectsStore } from "@/stores/projects";
import { useListProjectsQuery } from "@/services/api/projects.api";
import { useListSkillsQuery, useListTagsQuery } from "@/services/api/catalog.api";
import { useGetUnitConfigQuery } from "@/services/api/config.api";
import { cn } from "@/lib/utils";

const themeStore = useThemeStore();
const contentLayout = computed(() => themeStore.contentLayout);

const projects = useProjectsStore();

// Bootstrap queries — gate router-view readiness until core data is loaded.
const projectsQuery = useListProjectsQuery();
const skillsQuery = useListSkillsQuery();
const tagsQuery = useListTagsQuery();
const unitConfigQuery = useGetUnitConfigQuery();

const ready = computed(
  () =>
    projectsQuery.isSuccess.value &&
    skillsQuery.isSuccess.value &&
    tagsQuery.isSuccess.value &&
    unitConfigQuery.isSuccess.value,
);

const error = computed(() => {
  const queries = [projectsQuery, skillsQuery, tagsQuery, unitConfigQuery];
  const failed = queries.find((q) => q.isError.value);
  if (!failed) return null;
  const e = failed.error.value;
  return e instanceof Error ? e.message : String(e);
});

async function retry() {
  await Promise.all([
    projectsQuery.refetch(),
    skillsQuery.refetch(),
    tagsQuery.refetch(),
    unitConfigQuery.refetch(),
  ]);
}

// Auto-select the first project once data arrives.
watch(
  () => projectsQuery.data.value,
  (items) => {
    if (projects.current == null && items && items.length > 0) {
      projects.select(items[0].id);
    }
  },
);
</script>

<template>
  <SidebarProvider>
    <AppSidebar />

    <SidebarInset
      class="w-full max-w-full peer-data-[state=collapsed]:w-[calc(100%-var(--sidebar-width-icon)-1rem)] peer-data-[state=expanded]:w-[calc(100%-var(--sidebar-width))]"
    >
      <header
        class="flex items-center gap-3 sm:gap-4 h-14 p-4 shrink-0 border-b bg-card transition-[width,height] ease-linear"
      >
        <SidebarTrigger class="-ml-1" />
        <Separator orientation="vertical" class="h-6" />
        <CommandMenuPanel />
        <div class="flex-1" />
        <div class="ml-auto flex items-center space-x-4">
          <ToggleTheme />
          <ThemePopover />
        </div>
      </header>

      <main
        :class="
          cn(
            'p-6 grow overflow-auto',
            contentLayout === 'centered' ? 'container mx-auto' : '',
          )
        "
      >
        <div v-if="error" class="space-y-4">
          <Alert variant="destructive">
            <AlertTitle>加载失败</AlertTitle>
            <AlertDescription>{{ error }}</AlertDescription>
          </Alert>
          <Button variant="outline" @click="retry">重试</Button>
        </div>
        <div v-else-if="!ready" class="space-y-4">
          <Skeleton class="h-8 w-48" />
          <Skeleton class="h-32 w-full" />
          <Skeleton class="h-32 w-full" />
        </div>
        <router-view v-else />
      </main>
    </SidebarInset>
  </SidebarProvider>
</template>
