import { Component, createMemo, JSXElement } from "solid-js";
import { useUIClasses } from "../../stores/UIClasses.tsx";
import { useEngine } from "../../stores/engine.tsx";
import { Link } from "../../api_types/Link.ts";
import { useParams } from "@solidjs/router";
import { APINodeFlags } from "../../api_types/APINodeFlags.ts";
import { APINodeItem } from "../../api_types/APINodeItem.ts";
import ExternalLinkIcon from "../../assets/icons/heroicons-arrow-top-right-on-square.svg";
import ArrowPathIcon from "../../assets/icons/heroicons-arrow-path.svg";
import CheckIcon from "../../assets/icons/tabler-check.svg";
import CrossIcon from "../../assets/icons/tabler-cross.svg";
import ClockIcon from "../../assets/icons/tabler-clock.svg";
import HighlightTerms from "../generic/HighlightTerms.tsx";

interface ILinkPayloadProps {
  id: number;
  flags: Array<APINodeFlags>;
  payload: Link;
  showFlags: boolean;
  data?: Record<string, any>;
}

const SLOW_QUADRATIC_SPINNER_CLASS =
  "motion-safe:animate-[spin_2s_cubic-bezier(0.46,0.03,0.52,0.96)_infinite]";

const Payload: Component<ILinkPayloadProps> = (props) => {
  const [_engine, { getRelatedNodes, getRelatedNodeIds }] = useEngine();
  const [_, { getColors }] = useUIClasses();
  const params = useParams();

  const getDomain = createMemo<string | undefined>(() => {
    return getRelatedNodes(params.projectId, props.id, "BelongsTo", (node) =>
      node.labels.includes("Domain"),
    )[0]?.payload.data as string | undefined;
  });

  const getTitle = createMemo<string | undefined>(() => {
    const relatedContentNodeIds = getRelatedNodeIds(
      params.projectId,
      props.id,
      "PathOf",
    );
    if (relatedContentNodeIds.length === 0) {
      return undefined;
    }
    const titleNodes = getRelatedNodes(
      params.projectId,
      relatedContentNodeIds[0],
      "ParentOf",
      (node) => node.labels.includes("Title"),
    );

    return titleNodes[0]?.payload.data as string | undefined;
  });

  const getFullLink = createMemo<string>(() => {
    let fullPath = props.payload.path;
    if (!!props.payload.query) {
      fullPath += "?" + props.payload.query;
    }
    return "https://" + getDomain()! + fullPath;
  });

  const getStatusIcon = createMemo<JSXElement>(() => {
    const colorClass = props.flags.includes("IsProcessed")
      ? getColors().textSuccess
      : props.flags.includes("IsRequesting")
        ? getColors().textWarning
        : props.flags.includes("IsBlocked")
          ? getColors().textDanger
          : getColors().textMuted;
    return (
      <span
        class={`inline-block size-4 ${colorClass}`}
        classList={{
          [SLOW_QUADRATIC_SPINNER_CLASS]: props.flags.includes("IsRequesting"),
        }}
      >
        {props.flags.includes("IsProcessed") ? (
          <CheckIcon />
        ) : props.flags.includes("IsRequesting") ? (
          <ArrowPathIcon />
        ) : props.flags.includes("IsBlocked") ? (
          <CrossIcon />
        ) : (
          <ClockIcon />
        )}
      </span>
    );
  });

  return (
    <>
      {!!getDomain() ? (
        <div
          class="grid gap-2 mb-0.5 justify-start"
          classList={{
            "grid-cols-[30px_1fr]": props.showFlags,
            "grid-cols-1": !props.showFlags,
          }}
        >
          {props.showFlags && (
            <div class="text-center h-full">{getStatusIcon()}</div>
          )}
          <div class="text-sm w-full">
            <a
              href={getFullLink()}
              class={`${getColors().link}`}
              target="_blank"
              title={getFullLink()}
              rel="noopener noreferrer"
            >
              <span class="inline-block max-w-[90%] truncate">
                {props.data?.highlightTerms && (
                  <HighlightTerms
                    terms={props.data?.highlightTerms || []}
                    content={getTitle() || getFullLink()}
                  />
                )}
                {!props.data?.highlightTerms && (getTitle() || getFullLink())}
              </span>
              <span class="inline-block size-3 ml-0.5 mb-1">
                <ExternalLinkIcon />
              </span>
            </a>
            {!!getTitle() && (
              <div
                class={`text-nowrap truncate max-w-[80%] mb-1.5 ${getColors()["textMuted"]}`}
                title={getFullLink()}
              >
                {getFullLink()}
              </div>
            )}
          </div>
        </div>
      ) : (
        <></>
      )}
    </>
  );
};

interface ILinkNodeProps {
  nodeId: number;
  showFlags: boolean;
  data?: Record<string, any>;
}

const LinkNode: Component<ILinkNodeProps> = (props) => {
  const [_, { getNodeById }] = useEngine();
  const params = useParams();

  const getNode = createMemo(() => {
    const nodeByNodeId = getNodeById(params.projectId, props.nodeId);
    if (!!nodeByNodeId && nodeByNodeId.payload.type === "Link") {
      return nodeByNodeId as APINodeItem;
    }
    return undefined;
  });

  return (
    <>
      {!!getNode() ? (
        <>
          <Payload
            id={props.nodeId}
            flags={getNode()!.flags}
            payload={getNode()!.payload.data as Link}
            showFlags={props.showFlags}
            data={props.data}
          />
        </>
      ) : null}
    </>
  );
};

export default LinkNode;
