import type { App } from "vue";
import { setupAutoAnimate } from "./auto-animate/setup";
import { setupDayjs } from "./dayjs/setup";
import { setupI18n } from "./i18n/setup";
import { setupNProgress } from "./nprogress/setup";
import { setupPinia } from "./pinia/setup";
import { setupRouter } from "./router/setup";
import { setupTanstackVueQuery } from "./tanstack-vue-query/setup";

export function setupPlugins(app: App) {
  // 顺序：无依赖的纯配置先行；pinia 必须先于任何用到 store 的插件（router guard 将用 pinia）
  setupDayjs();
  setupNProgress();
  setupAutoAnimate(app);
  setupTanstackVueQuery(app);
  setupI18n(app);
  setupPinia(app);
  setupRouter(app);
}
