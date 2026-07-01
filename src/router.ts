import { createRouter, createWebHashHistory } from "vue-router";
import KanbanView from "./views/KanbanView.vue";
import ProjectsView from "./views/ProjectsView.vue";
import ResourcesView from "./views/ResourcesView.vue";

export const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: "/", redirect: "/kanban" },
    { path: "/kanban", component: KanbanView },
    { path: "/projects", component: ProjectsView },
    { path: "/resources", component: ResourcesView },
    { path: "/catalog", component: () => import("./views/CatalogView.vue") },
    { path: "/dashboard", component: () => import("./views/DashboardView.vue") },
    { path: "/allocations", component: () => import("./views/AllocationsView.vue") },
    { path: "/calendar", component: () => import("./views/CalendarView.vue") },
    { path: "/gantt", component: () => import("./views/GanttView.vue") },
    { path: "/calendar-grid", component: () => import("./views/CalendarGridView.vue") },
    { path: "/teams", component: () => import("./views/TeamsView.vue") },
    { path: "/ai", component: () => import("./views/AiPanelView.vue") },
    { path: "/reports", component: () => import("./views/ReportsView.vue") },
    { path: "/settings", component: () => import("./views/SettingsView.vue") },
  ],
});