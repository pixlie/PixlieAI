import { Component, createMemo, JSXElement } from "solid-js";
import { useUIClasses } from "../../stores/UIClasses.tsx";
import { useEngine } from "../../stores/engine.tsx";
import { Link } from "../../api_types/Link.ts";
import { useParams } from "@solidjs/router";
import { APINodeFlags } from "../../api_types/APINodeFlags.ts";
import { APINodeItem } from "../../api_types/APINodeItem.ts";
import { NodeLabel } from "../../api_types/NodeLabel.ts";
import ExternalLinkIcon from "../../assets/icons/heroicons-arrow-top-right-on-square.svg";
import ArrowPathIcon from "../../assets/icons/heroicons-arrow-path.svg";
import CheckIcon from "../../assets/icons/tabler-check.svg";
import CrossIcon from "../../assets/icons/tabler-cross.svg";
import ClockIcon from "../../assets/icons/tabler-clock.svg";

interface ILinkPayloadProps {
  id: number;
  flags: Array<APINodeFlags>;
  payload: Link;
  showFlags: boolean;
}

const SLOW_QUADRATIC_SPINNER_CLASS =
  "motion-safe:animate-[spin_2s_cubic-bezier(0.46,0.03,0.52,0.96)_infinite]";

const Payload: Component<ILinkPayloadProps> = (props) => {
  const [_engine, { getRelatedNodes }] = useEngine();
  const [_, { getColors }] = useUIClasses();
  const params = useParams();

  const getDomainNode = createMemo<APINodeItem | undefined>(() => {
    const relatedDomains = getRelatedNodes(
      params.projectId,
      props.id,
      "BelongsTo",
    );
    if (relatedDomains.length > 0) {
      if (relatedDomains[0].labels.includes("Domain" as NodeLabel)) {
        return relatedDomains[0];
      }
    }
    return undefined;
  });

  const getDomain = createMemo<string | undefined>(() => {
    return getDomainNode()?.payload.data as string;
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

  const getFullLink = createMemo<string>(() => {
    let fullPath = props.payload.path;
    if (!!props.payload.query) {
      fullPath += "?" + props.payload.query;
    }
    return 'https://' + getDomain()! + fullPath;
  });

  const isScraped = createMemo<boolean>(() => {
    const contentNodes = getRelatedNodes(
      params.projectId,
      props.id,
      "PathOf",
    );
    if (contentNodes.length === 0) {
      return false;
    }
    if (
      contentNodes[0].labels.includes("WebPage") &&
      contentNodes[0].flags.includes("IsProcessed")
    ) {
      return true;
    }
    return false;
  });

  // TODO: Remove derived node flag once backend bug
  // for saving node flags is fixed
  const getDerivedNodeFlag = createMemo<APINodeFlags>(() => {
    if (props.flags.includes("IsRequesting")) {
      return "IsRequesting";
    } else if (
      props.flags.includes("IsProcessed") ||
      getTitle() ||
      isScraped()
    ) {
      return "IsProcessed";
    } else if (props.flags.includes("IsBlocked")) {
      return "IsBlocked";
    }
    return "None";
  });

  const getStatusIcon = createMemo<JSXElement>(() => {
    const derivedNodeFlag = getDerivedNodeFlag();
    const colorClass  = {
      IsProcessed: getColors()["textSuccess"],
      IsRequesting: getColors()["textWarning"],
      IsBlocked: getColors()["textDanger"],
      None: getColors()["textMuted"],
    }[derivedNodeFlag];
    // const colorClass = props.flags.includes("IsProcessed")
    //   ? getColors()["textSuccess"]
    //   : props.flags.includes("IsRequesting")
    //   ? getColors()["textWarning"]
    //   : props.flags.includes("IsBlocked")
    //   ? getColors()["textDanger"]
    //   : props.flags.includes("None")
    //   ? getColors()["textMuted"]
    //   : "";
    return (
      <span
        class={`inline-block size-4 ${colorClass}`}
        classList={{
          // [SLOW_QUADRATIC_SPINNER_CLASS]: props.flags.includes("IsRequesting")
          [SLOW_QUADRATIC_SPINNER_CLASS]: derivedNodeFlag === "IsRequesting",
        }}
      >
        {{
          IsProcessed: <CheckIcon />,
          IsRequesting: <ArrowPathIcon />,
          IsBlocked: <CrossIcon />,
          None: <ClockIcon />,
        }[derivedNodeFlag]}
        {/* {props.flags.includes("IsProcessed") ? (
          <CheckIcon />
        ) : props.flags.includes("IsRequesting") ? (
          <ArrowPathIcon />
        ) : props.flags.includes("IsBlocked") ? (
          <CrossIcon />
        ) : props.flags.includes("None") ? (
          <ClockIcon />
        ) : (
          ""
        )} */}
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
                {!!getTitle() ? getTitle() : getFullLink()}
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
