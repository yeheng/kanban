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
    { path: "/dashboard", component: () => import("./views/DashboardView.vue") },
    { path: "/allocations", component: () => import("./views/AllocationsView.vue") },
  ],
});