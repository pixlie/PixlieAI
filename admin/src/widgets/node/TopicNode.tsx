import { Component, createMemo } from "solid-js";
import { useEngine } from "../../stores/engine";
import { useParams } from "@solidjs/router";
import { IEngine, INodeItemDisplayProps } from "../../utils/types";
import { Topic } from "../../api_types/Topic";
import Heading from "../typography/Heading";
import { useUIClasses } from "../../stores/UIClasses";
import { utcStringToLocaleStringAgo } from "../../utils/api";

const TopicNode: Component<INodeItemDisplayProps> = (props) => {
  const [engine] = useEngine();
  const params = useParams();
  const [_, { getColors }] = useUIClasses();

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
          <div class="flex flex-col mb-3">
            <Heading size={5}>{getPayload()?.topic}</Heading>
            <small class={getColors()["textMuted"]}>Last processed: {
              utcStringToLocaleStringAgo(getPayload()?.last_processed)
            }</small>
          </div>
          <small>
          __ search terms discovered 
          </small>
        </>
      ) : null}
    </>
  );
};

export default TopicNode;
