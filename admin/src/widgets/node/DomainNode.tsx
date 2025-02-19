import { Component, createMemo } from "solid-js";
import { useUIClasses } from "../../stores/UIClasses.tsx";
import { Domain } from "../../api_types/Domain.ts";
import { useEngine } from "../../stores/engine.tsx";
import { useParams } from "@solidjs/router";

interface IDomainPayloadProps {
  payload: Domain;
}

const Payload: Component<IDomainPayloadProps> = (props) => {
  const [_, { getColors }] = useUIClasses();

  return (
    <>
      <a href={`https://${props.payload.name}`} class={getColors().link}>
        {props.payload.name}
      </a>
      <span>
        {props.payload.is_allowed_to_crawl ? "Can crawl" : "Cannot crawl"}
      </span>
    </>
  );
};

interface IDomainNodeProps {
  nodeId: number;
}

const DomainNode: Component<IDomainNodeProps> = (props) => {
  const [engine] = useEngine();
  const params = useParams();

  const getProject = createMemo(() => {
    if (!!params.projectId && params.projectId in engine.projects) {
      return engine.projects[params.projectId];
    }
    return undefined;
  });

  return (
    <>
      {getProject() &&
        props.nodeId in getProject()!.nodes &&
        getProject()!.nodes[props.nodeId].payload.type === "Domain" && (
          <Payload
            payload={getProject()!.nodes[props.nodeId].payload.data as Domain}
          />
        )}
    </>
  );
};

export default DomainNode;
