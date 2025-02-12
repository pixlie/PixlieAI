import { RouteSectionProps } from "@solidjs/router";
import { Component } from "solid-js";
import { EngineProvider } from "../../stores/engine.tsx";

const PerProjectWrapper: Component<RouteSectionProps> = (props) => (
  <EngineProvider>{props.children}</EngineProvider>
);

export default PerProjectWrapper;
