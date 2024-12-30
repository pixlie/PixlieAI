import { Route, RouteSectionProps } from "@solidjs/router";
import { Component } from "solid-js";
import Setup from "./settings/Setup";
import Heading from "../widgets/typography/Heading";

const SettingsWrapper: Component<RouteSectionProps> = (props) => (
  <>
    <Heading size={1}>Settings</Heading>
    <div class="mb-4" />
    {props.children}
  </>
);

const SettingsRoutes: Component = () => (
  <>
    <Route path="/setup" component={Setup} />
  </>
);

export { SettingsWrapper, SettingsRoutes };
