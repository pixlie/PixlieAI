import { RouteSectionProps } from "@solidjs/router";
import { Component } from "solid-js";
import ProjectLoader from "../../widgets/projects/Loader";

const PerProjectWrapper: Component<RouteSectionProps> = (props) => (
  <>
    <ProjectLoader />
    {props.children}
  </>
);

export default PerProjectWrapper;
