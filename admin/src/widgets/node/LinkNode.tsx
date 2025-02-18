import { Component, createMemo } from "solid-js";
import { useUIClasses } from "../../stores/UIClasses.tsx";
import { useEngine } from "../../stores/engine.tsx";
import { Link } from "../../api_types/Link.ts";
import { APINodeItem } from "../../api_types/APINodeItem.ts";

interface ILinkPayloadProps {
  payload: Link;
  domain?: APINodeItem;
}

const Payload: Component<ILinkPayloadProps> = (props) => {
  const [_, { getColors }] = useUIClasses();

  return (
    <>
      {props.domain && props.domain.payload.type === "Domain" ? (
        <span class="text-xs bg-gray-300 rounded px-2 py-0.5">
          {props.domain.payload["data"].name}
        </span>
      ) : (
        <span></span>
      )}
      <a
        href={`${props.payload.path}${!!props.payload.query ? "?" + props.payload.query : ""}`}
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
  const [engine, { getRelatedNodes }] = useEngine();

  const getDomain = createMemo(() => {
    let relatedDomains = getRelatedNodes(props.nodeId, "BelongsTo");
    if (relatedDomains.length > 0) {
      return relatedDomains[0];
    }
  });

  return (
    <>
      {props.nodeId in engine.nodes &&
      engine.nodes[props.nodeId].payload.type === "Link" ? (
        <Payload
          payload={engine.nodes[props.nodeId].payload.data as Link}
          domain={getDomain()}
        />
      ) : null}
    </>
  );
};

export default LinkNode;
