import { create } from "zustand";

export type ThemeChoice = "light" | "dark" | "system";

const STORAGE_KEY = "sp.theme";
const CHOICES: readonly string[] = ["light", "dark", "system"];

function isChoice(v: unknown): v is ThemeChoice {
  return typeof v === "string" && CHOICES.includes(v);
}

function readStored(): ThemeChoice {
  try {
    const v = localStorage.getItem(STORAGE_KEY);
    return isChoice(v) ? v : "system";
  } catch {
    return "system";
  }
}

export function resolveTheme(choice: ThemeChoice, systemDark: boolean): "light" | "dark" {
  if (choice === "system") return systemDark ? "dark" : "light";
  return choice;
}

function systemPrefersDark(): boolean {
  return typeof matchMedia === "function" && matchMedia("(prefers-color-scheme: dark)").matches;
}

export function applyTheme(choice: ThemeChoice): void {
  document.documentElement.dataset.theme = resolveTheme(choice, systemPrefersDark());
}

interface ThemeState {
  choice: ThemeChoice;
  setChoice: (c: ThemeChoice) => void;
}

export const useTheme = create<ThemeState>((set) => ({
  choice: readStored(),
  setChoice: (choice) => {
    try {
      localStorage.setItem(STORAGE_KEY, choice);
    } catch {
      /* storage unavailable: theme still applies for this session */
    }
    applyTheme(choice);
    set({ choice });
  },
}));
