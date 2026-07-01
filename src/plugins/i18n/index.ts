import { watch } from "vue";
import { useStorage } from "@vueuse/core";

export type Language = "zh" | "en";

export const SUPPORTED_LOCALES = new Set<Language>(["zh", "en"]);

/** kanban 默认中文（与模板的 'en' 默认相反） */
export const DEFAULT_LOCALE: Language = "zh";

export const appLocale = useStorage<Language>("app-locale", DEFAULT_LOCALE);

watch(
  appLocale,
  (newLocale) => {
    if (!SUPPORTED_LOCALES.has(newLocale)) {
      appLocale.value = DEFAULT_LOCALE;
    }
  },
  { immediate: true },
);
