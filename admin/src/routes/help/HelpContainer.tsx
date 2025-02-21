import { RouteSectionProps } from "@solidjs/router";
import { Component } from "solid-js";

const HelpContainer: Component<RouteSectionProps> = (props) => (
  <>
    {props.children}
  </>
);

export default HelpContainer;
