import type { Component } from "vue";
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

export interface NavItem {
  title: string;
  url: string;
  icon: Component;
}

export function useAppNav(): { items: NavItem[] } {
  const items: NavItem[] = [
    { title: "Dashboard", url: "/dashboard", icon: LayoutDashboardIcon },
    { title: "Kanban", url: "/kanban", icon: KanbanIcon },
    { title: "Projects", url: "/projects", icon: FolderKanbanIcon },
    { title: "Resources", url: "/resources", icon: UsersIcon },
    { title: "Teams", url: "/teams", icon: UsersRoundIcon },
    { title: "Allocations", url: "/allocations", icon: ListChecksIcon },
    { title: "Calendar", url: "/calendar", icon: CalendarIcon },
    { title: "Gantt", url: "/gantt", icon: BarChart3Icon },
    { title: "Calendar Grid", url: "/calendar-grid", icon: LayoutGridIcon },
    { title: "Catalog", url: "/catalog", icon: TagsIcon },
    { title: "AI Optimization", url: "/ai", icon: SparklesIcon },
    { title: "Reports", url: "/reports", icon: FileTextIcon },
    { title: "Settings", url: "/settings", icon: SettingsIcon },
  ];
  return { items };
}
