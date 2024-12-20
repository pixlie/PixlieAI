interface Route {
  label: string;
  icon: string;
  href: string;
}

const routes = [
  {
    label: "Insights",
    icon: "insight",
    href: "/p/1/insights",
  },
  {
    label: "Graph",
    icon: "graph",
    href: "/p/1/graph",
  },
  {
    label: "Crawl",
    icon: "inbox",
    href: "/p/1/crawl",
  },
  {
    label: "Settings",
    icon: "cog",
    href: "/p/1/settings",
  },
];

export type { Route };
export { routes };
