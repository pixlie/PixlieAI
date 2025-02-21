import { RouteSectionProps } from "@solidjs/router";
import { Component } from "solid-js";

const SettingsContainer: Component<RouteSectionProps> = (props) => (
  <>
    {props.children}
  </>
);

export default SettingsContainer;
