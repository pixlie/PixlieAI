import { Component, createMemo } from "solid-js";
import { useUIClasses } from "../../stores/UIClasses.tsx";
import { Domain } from "../../api_types/Domain.ts";
import { useEngine } from "../../stores/engine.tsx";
import { useParams } from "@solidjs/router";
import { IEngine } from "../../utils/types.tsx";

interface IDomainNodeProps {
  nodeId: number;
}

const DomainNode: Component<IDomainNodeProps> = (props) => {
  const [engine] = useEngine();
  const params = useParams();
  const [_, { getColors }] = useUIClasses();

  const getProject = createMemo<IEngine | undefined>(() => {
    if (!!params.projectId && params.projectId in engine.projects) {
      return engine.projects[params.projectId];
    }
    return undefined;
  });

  const getPayload = createMemo<Domain | undefined>(() => {
    if (getProject() && props.nodeId in getProject()!.nodes) {
      return getProject()!.nodes[props.nodeId].payload.data as Domain;
    }
    return undefined;
  });

  return (
    <>
      {!!getPayload() ? (
        <>
          <a href={`https://${getPayload()!.name}`} class={getColors().link}>
            {getPayload()!.name}
          </a>
          <span class="text-xs">
            {getPayload()!.is_allowed_to_crawl ? "Can crawl" : "Cannot crawl"}
          </span>
        </>
      ) : null}
    </>
  );
};

export default DomainNode;
