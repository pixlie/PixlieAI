import { Component, createMemo } from "solid-js";
import { useParams } from "@solidjs/router";
import { useEngine } from "../../stores/engine.tsx";

const NodeEditor: Component = () => {
  // The node editor is created with free positioned divs that can be dragged and dropped
  // Each node's position can be saved (later). nodes are connected with edges which are SVG paths
  const [engine] = useEngine();
  const params = useParams();

  const getProject = createMemo(() => {
    if (!!params.projectId && params.projectId in engine.projects) {
      return engine.projects[params.projectId];
    }
    return undefined;
  });

  return <div class="relative"></div>;
};

export default NodeEditor;
