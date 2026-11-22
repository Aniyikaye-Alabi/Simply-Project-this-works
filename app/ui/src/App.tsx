import { useEffect } from "react";
import { t } from "./i18n";
import { applyTheme, useTheme, type ThemeChoice } from "./theme/themeStore";

const OPTIONS: { value: ThemeChoice; label: () => string }[] = [
  { value: "light", label: () => t("theme.light") },
  { value: "dark", label: () => t("theme.dark") },
  { value: "system", label: () => t("theme.system") },
];

export default function App() {
  const { choice, setChoice } = useTheme();

  useEffect(() => {
    applyTheme(choice);
    if (choice !== "system" || typeof matchMedia !== "function") return;
    const mq = matchMedia("(prefers-color-scheme: dark)");
    const onChange = () => applyTheme("system");
    mq.addEventListener("change", onChange);
    return () => mq.removeEventListener("change", onChange);
  }, [choice]);

  return (
    <main className="min-h-screen bg-bg p-4 text-text">
      <header className="flex items-center justify-between rounded-xl bg-surface p-4 shadow-[var(--shadow)]">
        <h1 className="text-xl font-semibold">{t("app.name")}</h1>
        <label className="flex items-center gap-2">
          <span className="text-text-muted">{t("theme.label")}</span>
          <select
            className="min-h-10 rounded-[10px] border border-border bg-surface-2 px-3 text-base"
            value={choice}
            onChange={(e) => setChoice(e.target.value as ThemeChoice)}
          >
            {OPTIONS.map((o) => (
              <option key={o.value} value={o.value}>
                {o.label()}
              </option>
            ))}
          </select>
        </label>
      </header>
      <p className="mt-6 text-center text-text-muted">{t("shell.empty")}</p>
    </main>
  );
}
