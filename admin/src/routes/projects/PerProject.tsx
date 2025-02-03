import { RouteSectionProps } from "@solidjs/router";
import { Component } from "solid-js";

const PerProjectWrapper: Component<RouteSectionProps> = (props) => (
  <>{props.children}</>
);

export default PerProjectWrapper;
