import { RouteSectionProps } from "@solidjs/router";
import { Component } from "solid-js";
import Heading from "../widgets/typography/Heading";

const SettingsContainer: Component<RouteSectionProps> = (props) => (
  <>
    <Heading size={1}>Settings</Heading>
    <div class="mb-4" />
    {props.children}
  </>
);

export default SettingsContainer;
