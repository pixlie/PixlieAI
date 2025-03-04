import { RouteSectionProps, useParams } from "@solidjs/router";
import { Component, createEffect, JSX, onMount } from "solid-js";
import { EngineProvider, useEngine } from "../../stores/engine.tsx";

interface IPerProjectInnerProps {
  children: JSX.Element;
}

const PerProjectInner: Component<IPerProjectInnerProps> = (props) => {
  const [_, { setProjectId, fetchNodes, fetchAllEdges }] = useEngine();
  const params = useParams();

  onMount(() => {
    setProjectId(params.projectId);
    fetchNodes(params.projectId);
    fetchAllEdges(params.projectId);
  });

  createEffect(() => {
    if (params.projectId) {
      setProjectId(params.projectId);
      fetchNodes(params.projectId);
      fetchAllEdges(params.projectId);
    }
  });

  return <>{props.children}</>;
};

const PerProjectWrapper: Component<RouteSectionProps> = (props) => {
  return (
    <EngineProvider>
      <PerProjectInner>{props.children}</PerProjectInner>
    </EngineProvider>
  );
};

export default PerProjectWrapper;
