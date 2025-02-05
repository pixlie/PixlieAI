interface Route {
  label: string;
  icon: string;
  href: string;
}

const globalRoutes = [
  {
    label: "Projects",
    href: "/p",
  },
];

const perProjectRoutes = [
  {
    label: "Workflow",
    href: "/workflow",
  },
  {
    label: "Insights",
    href: "/insights",
  },
  {
    label: "Graph",
    href: "/graph",
  },
  {
    label: "Crawl",
    href: "/crawl",
  },
];

export type { Route };
export { globalRoutes, perProjectRoutes };
