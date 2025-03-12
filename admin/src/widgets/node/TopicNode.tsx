import { Component, createMemo } from "solid-js";
import { useEngine } from "../../stores/engine";
import { useParams } from "@solidjs/router";
import { IEngine, INodeItemDisplayProps } from "../../utils/types";
import Heading from "../typography/Heading";

const TopicNode: Component<INodeItemDisplayProps> = (props) => {
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
          <div class="flex flex-col mb-3">
            <Heading size={5}>{getPayload()}</Heading>
          </div>
          <small>__ search terms discovered</small>
        </>
      ) : null}
    </>
  );
};

export default TopicNode;
