import en from "./en.json";

export type StringKey = keyof typeof en;

/** All user-facing copy goes through here (SET-6). English only for now. */
export function t(key: StringKey): string {
  return en[key];
}
