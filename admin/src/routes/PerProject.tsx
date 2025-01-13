import { Route, RouteSectionProps } from "@solidjs/router";
import { Component } from "solid-js";
import Insights from "./project/Insights";
import Graph from "./project/Graph";
import Crawl from "./project/Crawl";

const PerProjectWrapper: Component<RouteSectionProps> = (props) => (
  <>{props.children}</>
);

const PerProjectRoutes: Component = () => (
  <>
    <Route path="/:id/insights" component={Insights} />
    <Route path="/:id/graph" component={Graph} />
    <Route path="/:id/crawl" component={Crawl} />
  </>
);

export { PerProjectWrapper, PerProjectRoutes };
