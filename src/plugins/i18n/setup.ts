import type { App } from "vue";
import { createI18n } from "vue-i18n";

import { appLocale, DEFAULT_LOCALE } from ".";
import type { Language } from ".";
import en from "./en.json";
import zh from "./zh.json";

export function setupI18n(app: App) {
  const i18n = createI18n({
    legacy: false,
    locale: appLocale.value,
    fallbackLocale: DEFAULT_LOCALE,
    messages: {
      zh,
      en,
    } as Record<Language, Record<string, any>>,
  });
  app.use(i18n);
}
