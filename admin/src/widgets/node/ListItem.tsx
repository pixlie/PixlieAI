import { Component } from "solid-js";
import { useEngine } from "../../stores/engine";
import { Payload } from "../../api_types/Payload";
import LinkNode from "./LinkNode.tsx";
import DomainNode from "./DomainNode.tsx";

interface NodePayloadProps {
  payload: Payload;
}

const NodePayload: Component<NodePayloadProps> = (props) => {
  return (
    <>
      {"Link" in props.payload && <LinkNode {...props.payload["Link"]} />}
      {"Domain" in props.payload && <DomainNode {...props.payload["Domain"]} />}
      {"Title" in props.payload && (
        <div class="mb-2">{props.payload["Title"]}</div>
      )}
      {"Paragraph" in props.payload && (
        <div class="mb-6">{props.payload["Paragraph"]}</div>
      )}
      {"Heading" in props.payload && (
        <div class="mb-2">{props.payload["Heading"]}</div>
      )}
      {"BulletPoints" in props.payload && (
        <div class="mb-6">
          {props.payload["BulletPoints"].map((x) => (
            <div class="list-item">{x}</div>
          ))}
        </div>
      )}
      {"OrderedPoints" in props.payload && (
        <div class="mb-6">
          {props.payload["OrderedPoints"].map((x) => (
            <div class="list-item">{x}</div>
          ))}
        </div>
      )}
    </>
  );
};

interface NodeListItemProps {
  nodeId: number;
}

const NodeListItem: Component<NodeListItemProps> = (props) => {
  const [engine] = useEngine();

  return (
    <>
      {engine.isReady && props.nodeId in engine.nodes ? (
        <NodePayload payload={engine.nodes[props.nodeId].payload} />
      ) : (
        <></>
      )}
    </>
  );
};

export default NodeListItem;
