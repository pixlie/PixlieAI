import { Component, createMemo } from "solid-js";
import { useUIClasses } from "../../stores/UIClasses.tsx";
import { useEngine } from "../../stores/engine.tsx";
import { useParams } from "@solidjs/router";
import { IEngine } from "../../utils/types.tsx";
import { NodeLabel } from "../../api_types/NodeLabel.ts";

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

  const getPayload = createMemo<string | undefined>(() => {
    if (getProject() && props.nodeId in getProject()!.nodes) {
      let node = getProject()!.nodes[props.nodeId];
      if (
        node.labels.includes("Domain" as NodeLabel) &&
        node.payload.type === "Text"
      ) {
        return node.payload.data as string;
      }
      return undefined;
    }
    return undefined;
  });

  return (
    <>
      {!!getPayload() ? (
        <div class="flex items-center gap-5">
          <a
            class={getColors().link}
            href={`https://${getPayload()!}`}
            target="_blank"
            rel="noreferrer"
          >
            {getPayload()!}
          </a>
        </div>
      ) : null}
    </>
  );
};

export default DomainNode;
