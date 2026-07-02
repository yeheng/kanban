import { createApp } from "vue";
import App from "./App.vue";
import { setupPlugins } from "./plugins";
import "@/utils/env";
import "./styles.css";
import "./styles/themes.css";

function bootstrap() {
  const app = createApp(App);
  setupPlugins(app);
  app.mount("#app");
}

bootstrap();
