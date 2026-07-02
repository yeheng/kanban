import { storeToRefs } from "pinia";
import { watch } from "vue";

import { THEMES } from "@/constants/themes";
import { useThemeStore } from "@/stores/theme";

export function useSystemTheme() {
  const themeStore = useThemeStore();
  const { setTheme, setRadius } = themeStore;
  const { theme, radius } = storeToRefs(themeStore);

  if (typeof document !== "undefined") {
    watch(
      theme,
      (t) => {
        document.documentElement.classList.remove(...THEMES.map((x) => `theme-${x}`));
        document.documentElement.classList.add(`theme-${t}`);
      },
      { immediate: true },
    );

    watch(
      radius,
      (r) => {
        document.documentElement.style.setProperty("--radius", `${r}rem`);
      },
      { immediate: true },
    );
  }

  return {
    theme,
    radius,
    setTheme,
    setRadius,
  };
}
