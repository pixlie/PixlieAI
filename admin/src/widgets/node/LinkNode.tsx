import { Component, createMemo } from "solid-js";
import { useUIClasses } from "../../stores/UIClasses.tsx";
import { useEngine } from "../../stores/engine.tsx";
import { Link } from "../../api_types/Link.ts";
import { Domain } from "../../api_types/Domain.ts";
import { useParams } from "@solidjs/router";
import { APINodeFlags } from "../../api_types/APINodeFlags.ts";
import { APINodeItem } from "../../api_types/APINodeItem.ts";

interface ILinkPayloadProps {
  id: number;
  flags: Array<APINodeFlags>;
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
      {!!getDomain() && getDomain()!.is_allowed_to_crawl ? (
        <>
          <span>
            <span
              class="w-[20px] inline-block text-center mr-2"
              classList={{
                [getColors()["textSuccess"]]: props.flags.includes(
                  "IsProcessed" as APINodeFlags
                ),
                [getColors()["textMuted"]]: !props.flags.includes(
                  "IsProcessed" as APINodeFlags
                ),
              }}
            >
              {props.flags.includes("IsProcessed" as APINodeFlags) ? "✓" : ""}
              {props.flags.includes("IsRequesting" as APINodeFlags) ? "⌛" : ""}
            </span>
            <span class="text-xs bg-gray-300 rounded px-2 py-0.5">
              {getDomain()!.name}
            </span>
          </span>
          <a
            href={`https://${!!getDomain() ? getDomain()!.name : ""}${props.payload.path}${!!props.payload.query ? "?" + props.payload.query : ""}`}
            class={
              "text-sm text-nowrap overflow-hidden text-ellipsis " +
              getColors().link
            }
            target="_blank"
          >
            {`${props.payload.path}${!!props.payload.query ? "?" + props.payload.query : ""}`}
          </a>
          <span></span>
        </>
      ) : (
        <></>
      )}
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

  const getNode = createMemo(() => {
    if (
      !!getProject() &&
      props.nodeId in getProject()!.nodes &&
      getProject()!.nodes[props.nodeId].payload.type === "Link"
    ) {
      return getProject()!.nodes[props.nodeId] as APINodeItem;
    }
    return undefined;
  });

  return (
    <>
      {!!getProject() && !!getNode() ? (
        <Payload
          id={props.nodeId}
          flags={getNode()!.flags}
          payload={getNode()!.payload.data as Link}
        />
      ) : null}
    </>
  );
};

export default LinkNode;
