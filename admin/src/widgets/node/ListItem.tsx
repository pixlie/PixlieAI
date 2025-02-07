import { Component } from "solid-js";
import { useEngine } from "../../stores/engine";
import { Payload } from "../../api_types/Payload";
import { useUIClasses } from "../../stores/UIClasses.tsx";

interface NodePayloadProps {
  payload: Payload;
}

const NodePayload: Component<NodePayloadProps> = (props) => {
  const [_, { getColors }] = useUIClasses();

  return (
    <>
      {"Link" in props.payload && (
        <div class="grid grid-cols-3 items-center">
          <div>
            <a href={props.payload["Link"].url} class={getColors().link}>
              {props.payload["Link"].url}
            </a>
          </div>
          <div>
            {props.payload["Link"].is_fetched ? "Fetched" : "Not Fetched"}
          </div>
        </div>
      )}
      {"Domain" in props.payload && (
        <div>
          <a
            href={`https://${props.payload["Domain"].name}`}
            class={getColors().link}
          >
            {props.payload["Domain"].name}
          </a>
        </div>
      )}
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
      {!!engine.isReady && props.nodeId in engine.nodes ? (
        <NodePayload payload={engine.nodes[props.nodeId].payload} />
      ) : (
        <></>
      )}
    </>
  );
};

export default NodeListItem;
