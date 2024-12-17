import { Route, RouteSectionProps } from "@solidjs/router";
import { Component } from "solid-js";

import Insights from "../screens/Insights";
import Graph from "../screens/Graph";
import Crawl from "../screens/Crawl";
import Settings from "../screens/Settings";

const PerProjectWrapper: Component<RouteSectionProps> = (props) => (
  <>{props.children}</>
);

const PerProjectRoutes: Component = () => (
  <>
    <Route path="/:id/insights" component={Insights} />
    <Route path="/:id/graph" component={Graph} />
    <Route path="/:id/crawl" component={Crawl} />
    <Route path="/:id/settings" component={Settings} />
  </>
);

export { PerProjectWrapper, PerProjectRoutes };
