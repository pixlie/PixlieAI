import { Component, createMemo } from "solid-js";
import { useEngine } from "../../stores/engine";
import { useParams } from "@solidjs/router";
import { IEngine, INodeItemDisplayProps } from "../../utils/types";
import { Topic } from "../../api_types/Topic";

const TopicNode: Component<INodeItemDisplayProps> = (props) => {
  const [engine] = useEngine();
  const params = useParams();

  const getProject = createMemo<IEngine | undefined>(() => {
    if (!!params.projectId && params.projectId in engine.projects) {
      return engine.projects[params.projectId];
    }
    return undefined;
  });

  const getPayload = createMemo<Topic | undefined>(() => {
    if (getProject() && props.nodeId in getProject()!.nodes) {
      return getProject()!.nodes[props.nodeId].payload.data as Topic;
    }
    return undefined;
  });

  return (
    <>
      {!!getPayload() ? (
        <>
          <span>{getPayload()!}</span>
          <span>Search Terms:</span>
        </>
      ) : null}
    </>
  );
};

export default TopicNode;
