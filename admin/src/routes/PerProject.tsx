import { Route, RouteSectionProps } from "@solidjs/router";
import { Component } from "solid-js";
import Insights from "./project/Insights";
import Graph from "./project/Graph";
import Crawl from "./project/Crawl";
import { EngineProvider } from "../stores/engine";

const PerProjectWrapper: Component<RouteSectionProps> = (props) => (
  <>{props.children}</>
);

const PerProjectRoutes: Component = () => (
  <EngineProvider>
    <Route path="/:id/insights" component={Insights} />
    <Route path="/:id/graph" component={Graph} />
    <Route path="/:id/crawl" component={Crawl} />
  </EngineProvider>
);

export { PerProjectWrapper, PerProjectRoutes };
