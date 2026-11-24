import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";
import App from "../App";
import { resolveTheme, useTheme } from "../theme/themeStore";
import en from "../i18n/en.json";

describe("theme (SET-1)", () => {
  it("set_1_resolves_system_choice_from_os_preference", () => {
    expect(resolveTheme("system", true)).toBe("dark");
    expect(resolveTheme("system", false)).toBe("light");
    expect(resolveTheme("light", true)).toBe("light");
    expect(resolveTheme("dark", false)).toBe("dark");
  });

  it("set_1_switching_applies_instantly_and_persists", async () => {
    render(<App />);
    await userEvent.selectOptions(screen.getByLabelText(en["theme.label"]), "dark");
    expect(document.documentElement.dataset.theme).toBe("dark");
    expect(localStorage.getItem("sp.theme")).toBe("dark");
    await userEvent.selectOptions(screen.getByLabelText(en["theme.label"]), "light");
    expect(document.documentElement.dataset.theme).toBe("light");
  });

  it("set_1_survives_unavailable_storage", () => {
    const orig = Storage.prototype.setItem;
    Storage.prototype.setItem = () => {
      throw new Error("blocked");
    };
    try {
      expect(() => useTheme.getState().setChoice("dark")).not.toThrow();
      expect(document.documentElement.dataset.theme).toBe("dark");
    } finally {
      Storage.prototype.setItem = orig;
    }
  });
});

describe("copy rules (AGENTS.md section 2)", () => {
  const banned = /\b(scene|source|encoder|bitrate|codec|webrtc|stun|turn|dtls)\b/i;
  it("ui_strings_contain_no_banned_jargon", () => {
    for (const [key, value] of Object.entries(en)) {
      expect(value, key).not.toMatch(banned);
    }
  });
});
