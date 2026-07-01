import type { App } from "vue";
import { QueryClient, VueQueryPlugin } from "@tanstack/vue-query";
// NOTE: @tanstack/vue-query-devtools v6 exports VueQueryDevtools as a Vue
// *component* (not an installable plugin), so it can no longer be registered
// via app.use(). Devtools rendering is deferred to App.vue at wire-up (Task 5).

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 1000 * 60 * 5,
    },
  },
});

export function setupTanstackVueQuery(app: App) {
  app.use(VueQueryPlugin, { queryClient });
}
