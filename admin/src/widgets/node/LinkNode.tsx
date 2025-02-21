import { Component, createMemo } from "solid-js";
import { useUIClasses } from "../../stores/UIClasses.tsx";
import { useEngine } from "../../stores/engine.tsx";
import { Link } from "../../api_types/Link.ts";
import { Domain } from "../../api_types/Domain.ts";
import { useParams } from "@solidjs/router";

interface ILinkPayloadProps {
  id: number;
  payload: Link;
}

const Payload: Component<ILinkPayloadProps> = (props) => {
  const [_engine, { getRelatedNodes }] = useEngine();
  const [_, { getColors }] = useUIClasses();
  const params = useParams();

  const getDomain = createMemo<Domain | undefined>(() => {
    let relatedDomains = getRelatedNodes(
      params.projectId,
      props.id,
      "BelongsTo",
    );
    if (relatedDomains.length > 0) {
      if (relatedDomains[0].payload.type === "Domain") {
        return relatedDomains[0].payload.data as Domain;
      }
    }
    return undefined;
  });

  return (
    <>
      {!!getDomain() ? (
        <span class="text-xs bg-gray-300 rounded px-2 py-0.5">
          {getDomain()!.name}
        </span>
      ) : (
        <span></span>
      )}
      <a
        href={`${!!getDomain() ? getDomain()!.name : ""}${props.payload.path}${!!props.payload.query ? "?" + props.payload.query : ""}`}
        class={"text-sm text-nowrap " + getColors().link}
        target="_blank"
      >
        {`${props.payload.path}${!!props.payload.query ? "?" + props.payload.query : ""}`}
      </a>
      <span>{props.payload.is_fetched ? "Fetched" : "Not Fetched"}</span>
    </>
  );
};

interface ILinkNodeProps {
  nodeId: number;
}

const LinkNode: Component<ILinkNodeProps> = (props) => {
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
      getProject()!.nodes[props.nodeId].payload.type === "Link" ? (
        <Payload
          id={props.nodeId}
          payload={getProject()!.nodes[props.nodeId].payload.data as Link}
        />
      ) : null}
    </>
  );
};

export default LinkNode;
