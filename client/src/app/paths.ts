export const paths = {
  home: "/",
  licenses: "/licenses",
  manage: "/manage",
  manageShare: (id: string) => `/manage/${encodeURIComponent(id)}`,
  privacy: "/privacy",
  share: (id: string) => `/s/${encodeURIComponent(id)}`,
} as const;

export const routePatterns = {
  share: "/s/:id",
  manageShare: "/manage/:id",
} as const;
