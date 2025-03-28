import { Component, createMemo } from "solid-js";
import { useUIClasses } from "../../stores/UIClasses.tsx";
import { useEngine } from "../../stores/engine.tsx";
import { Link } from "../../api_types/Link.ts";
import { useParams } from "@solidjs/router";
import { APINodeFlags } from "../../api_types/APINodeFlags.ts";
import { APINodeItem } from "../../api_types/APINodeItem.ts";
import { NodeLabel } from "../../api_types/NodeLabel.ts";

interface ILinkPayloadProps {
  id: number;
  flags: Array<APINodeFlags>;
  payload: Link;
  showFlags: boolean;
}

const Payload: Component<ILinkPayloadProps> = (props) => {
  const [_engine, { getRelatedNodes }] = useEngine();
  const [_, { getColors }] = useUIClasses();
  const params = useParams();

  const getDomain = createMemo<string | undefined>(() => {
    const relatedDomains = getRelatedNodes(
      params.projectId,
      props.id,
      "BelongsTo",
    );
    if (relatedDomains.length > 0) {
      if (relatedDomains[0].labels.includes("Domain" as NodeLabel)) {
        return relatedDomains[0].payload.data as string;
      }
    }
    return undefined;
  });

  const getTitle = createMemo<string | null>(() => {
    const relatedContentNodes = getRelatedNodes(
      params.projectId,
      props.id,
      "PathOf",
    );
    if (relatedContentNodes.length === 0) {
      return null;
    }
    const titleNodes = getRelatedNodes(
      params.projectId,
      relatedContentNodes[0].id,
      "ParentOf"
    ).filter((node) => node.labels.includes("Title"));

    return titleNodes[0]?.payload.data as string;
  });

  const getFullLink = createMemo(() => {
    let fullPath = props.payload.path;
    if (!!props.payload.query) {
      fullPath += "?" + props.payload.query;
    }
    return 'https://' + getDomain()! + fullPath;
  });

  return (
    <>
      {!!getDomain() ? (
        <>
          <div class="flex items-start gap-2 mb-2">
            {props.showFlags && (
              <div
                class="flex-shrink text-center w-[20px]"
                classList={{
                  [getColors()["textSuccess"]]: props.flags.includes(
                    "IsProcessed" as APINodeFlags
                  ),
                  [getColors()["textMuted"]]: !props.flags.includes(
                    "IsProcessed" as APINodeFlags
                  ),
                }}
              >
                {props.flags.includes("IsProcessed") ? "✓" : ""}
                {props.flags.includes("IsRequesting") ? "⌛" : ""}
              </div>
            )}
            <div>
              <a
                href={getFullLink()}
                class={
                  "text-sm text-nowrap overflow-hidden text-ellipsis leading- " +
                  getColors().link
                }
                target="_blank"
                rel="noopener noreferrer"
              >
                {!!getTitle() ? getTitle() : getFullLink()}
              </a>
              {!!getTitle() && (
                <div
                  class={`text-sm ${getColors()["textMuted"]}`}
                >
                  {getFullLink()}
                </div>
              )}
            </div>
          </div>
        </>
      ) : (
        <></>
      )}
    </>
  );
};

interface ILinkNodeProps {
  nodeId: number;
  showFlags: boolean;
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
        <>
          <Payload
            id={props.nodeId}
            flags={getNode()!.flags}
            payload={getNode()!.payload.data as Link}
            showFlags={props.showFlags}
          />
        </>
      ) : null}
    </>
  );
};

export default LinkNode;
