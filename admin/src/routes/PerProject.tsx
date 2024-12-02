import { RouteSectionProps } from "@solidjs/router";
import { Component } from "solid-js";

const PerProjectWrapper: Component<RouteSectionProps> = (props) => (
  <>{props.children}</>
);

const PerProjectRoutes: Component = () => <>{/* <Route path="/:id" /> */}</>;

export { PerProjectWrapper, PerProjectRoutes };
