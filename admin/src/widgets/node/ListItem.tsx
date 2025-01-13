import { Component, createMemo } from "solid-js";
import { Node } from "../../api_types/Node";
import { useEngine } from "../../stores/engine";
import { Payload } from "../../api_types/Payload";

interface NodePayloadProps {
  payload: Payload;
}

const NodePayload: Component<NodePayloadProps> = (props) => {
  return (
    <>
      {"Link" in props.payload ? (
        <div class="grid grid-cols-3 items-center">
          <div>{props.payload["Link"].text}</div>
          <div>{props.payload["Link"].url}</div>
          <div>
            {props.payload["Link"].is_fetched ? "Fetched" : "Not Fetched"}
          </div>
        </div>
      ) : (
        <></>
      )}
    </>
  );
};

interface NodeListItemProps {
  nodeId: number;
}

const NodeListItem: Component<NodeListItemProps> = (props) => {
  const [engine] = useEngine();
  const getNode = createMemo<Node | undefined>(() =>
    engine.isReady && props.nodeId in engine.nodes
      ? engine.nodes[props.nodeId]
      : undefined,
  );

  return (
    <>
      {!!engine.isReady && !!getNode() ? (
        <NodePayload payload={getNode()!.payload} />
      ) : (
        <></>
      )}
    </>
  );
};

export default NodeListItem;
