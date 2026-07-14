export const navigationItems = [
  { id: "library", label: "提示词库" },
  { id: "inbox", label: "收件箱" },
  { id: "search", label: "搜索" },
  { id: "settings", label: "设置" },
] as const;

export type AppRoute = (typeof navigationItems)[number]["id"];
