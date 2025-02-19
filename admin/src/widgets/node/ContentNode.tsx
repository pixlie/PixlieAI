import { Component, createMemo } from "solid-js";
import { useEngine } from "../../stores/engine";
import { useParams } from "@solidjs/router";
import { IEngine, INodeItemDisplayProps } from "../../utils/types";
import { Title } from "../../api_types/Title";
import { Heading } from "../../api_types/Heading";
import { Paragraph } from "../../api_types/Paragraph";

const ContentNode: Component<INodeItemDisplayProps> = (props) => {
  const [engine] = useEngine();
  const params = useParams();

  const getProject = createMemo<IEngine | undefined>(() => {
    if (!!params.projectId && params.projectId in engine.projects) {
      return engine.projects[params.projectId];
    }
    return undefined;
  });

  const getPayload = createMemo<Title | Heading | Paragraph | undefined>(() => {
    if (getProject() && props.nodeId in getProject()!.nodes) {
      let payload = getProject()!.nodes[props.nodeId].payload;
      console.log(payload.type);
      if (payload.type === "Title") {
        return payload.data as Title;
      } else if (payload.type === "Heading") {
        return payload.data as Heading;
      } else if (payload.type === "Paragraph") {
        return payload.data as Paragraph;
      }
      return undefined;
    }
    return undefined;
  });

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
