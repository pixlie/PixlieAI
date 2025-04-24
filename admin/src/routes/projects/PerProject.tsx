import { RouteSectionProps, useParams } from "@solidjs/router";
import { Component, createEffect, JSX, onCleanup, onMount } from "solid-js";
import { EngineProvider, useEngine } from "../../stores/engine.tsx";

interface IPerProjectInnerProps {
  children: JSX.Element;
}

const PerProjectInner: Component<IPerProjectInnerProps> = (props) => {
  const [_, { setProjectId, sync, stopSync }] = useEngine();
  const params = useParams();

  onMount(() => {
    if (!!params.projectId) {
      setProjectId(params.projectId);
      // It is set to OFF temporarily
      // sync(params.projectId);
    }
  });

  createEffect((prevProjectId: string | void) => {
    if (!!params.projectId && prevProjectId !== params.projectId) {
      setProjectId(params.projectId);
      sync(params.projectId);
    }
    if (!!prevProjectId && prevProjectId !== params.projectId) {
      stopSync(prevProjectId);
    }
    return params.projectId;
  }, params.projectId);

  onCleanup(() => {
    stopSync();
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
