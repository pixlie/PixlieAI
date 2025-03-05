import { Component, createMemo } from "solid-js";
import { useEngine } from "../../stores/engine";
import { useParams } from "@solidjs/router";
import { INodeItemDisplayProps } from "../../utils/types";

const ContentNode: Component<INodeItemDisplayProps> = (props) => {
  const [engine] = useEngine();
  const params = useParams();

  const getProject = createMemo(() => {
    if (!!params.projectId && params.projectId in engine.projects) {
      return engine.projects[params.projectId];
    }
    return undefined;
  });

  const getPayload = createMemo<string | undefined>(() => {
    if (!!getProject() && props.nodeId in getProject()!.nodes) {
      let node = getProject()!.nodes[props.nodeId];

      if (node.payload.type === "Text") {
        return node.payload.data as string;
      }
      return undefined;
    }
    return undefined;
  });

  console.log(params.projectId, props.nodeId);

  return (
    <>
      {!!getPayload() ? (
        <>
          <span>{getPayload()!}</span>
        </>
      ) : null}
    </>
  );
};

export default ContentNode;
