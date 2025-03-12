import { Component, createMemo } from "solid-js";
import { useEngine } from "../../stores/engine";
import { useParams } from "@solidjs/router";
import { IEngine, INodeItemDisplayProps } from "../../utils/types";

const SearchTermNode: Component<INodeItemDisplayProps> = (props) => {
  const [engine] = useEngine();
  const params = useParams();

  const getProject = createMemo<IEngine | undefined>(() => {
    if (!!params.projectId && params.projectId in engine.projects) {
      return engine.projects[params.projectId];
    }
    return undefined;
  });

  const getPayload = createMemo<string | undefined>(() => {
    if (getProject() && props.nodeId in getProject()!.nodes) {
      return getProject()!.nodes[props.nodeId].payload.data as string;
    }
    return undefined;
  });

  return (
    <>
      {!!getPayload() ? (
        <>
          <span>{getPayload()!}</span>
          <span>Hits:</span>
        </>
      ) : null}
    </>
  );
};

export default SearchTermNode;
