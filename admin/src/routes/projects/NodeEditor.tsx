import { Component, createMemo, onMount } from "solid-js";
import { useParams } from "@solidjs/router";
import { useEngine } from "../../stores/engine.tsx";
import { getPixlieAIAPIRoot } from "../../utils/api.ts";

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

  const getNodes = createMemo(() => {
    // We use the describe endpoint of the backend to get nodes and edges
    if (!getProject()) {
      return;
    }

    let projectId = params.projectId;
    let pixlieAPIRoot = getPixlieAIAPIRoot();
    fetch(`${pixlieAPIRoot}/api/engine/${projectId}/describe`, {
      headers: {
        "Content-Type": "application/json",
      },
    })
      .then((response) => {
        if (!response.ok) {
          throw new Error("Failed to fetch nodes");
        }

        return response.json();
      })
      .then((data) => {
        console.log(data);
      });
  });

  onMount(() => {
    getNodes();
  });

  return <div class="relative"></div>;
};

export default NodeEditor;
