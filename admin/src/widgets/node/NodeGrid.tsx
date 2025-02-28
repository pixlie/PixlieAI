import { Component, For } from "solid-js";
import LinkNode from "./LinkNode";
import DomainNode from "./DomainNode";
import SearchTermNode from "./SearchTermNode";
import ContentNode from "./ContentNode";
import Heading from "../typography/Heading";

// interface NodePayloadProps {
//   id: number;
//   payload: APIPayload;
// }

// const NodePayload: Component<NodePayloadProps> = (props) => {
//   return (
//     <>
//       {"Domain" in props.payload && <DomainNode {...props.payload["Domain"]} />}
//       {"Title" in props.payload && (
//         <div class="mb-2">{props.payload["Title"]}</div>
//       )}
//       {"Paragraph" in props.payload && (
//         <div class="mb-6">{props.payload["Paragraph"]}</div>
//       )}
//       {"Heading" in props.payload && (
//         <div class="mb-2">{props.payload["Heading"]}</div>
//       )}
//       {"BulletPoints" in props.payload && (
//         <div class="mb-6">
//           {props.payload["BulletPoints"].map((x) => (
//             <div class="list-item">{x}</div>
//           ))}
//         </div>
//       )}
//       {"OrderedPoints" in props.payload && (
//         <div class="mb-6">
//           {props.payload["OrderedPoints"].map((x) => (
//             <div class="list-item">{x}</div>
//           ))}
//         </div>
//       )}
//     </>
//   );
// };

interface NodeListItemProps {
  nodeType?: string;
  source: () => Array<number>;
}

const NodeGrid: Component<NodeListItemProps> = (props) => {
  return (
    <>
      {props.nodeType ? (
        <>
          <Heading size={3}>{props.nodeType}s</Heading>
          {props.nodeType === "Link" && (
            <div class="grid grid-cols-[auto_1fr_auto] gap-2">
              <For each={props.source()}>
                {(nodeId) => <LinkNode nodeId={nodeId} />}
              </For>
            </div>
          )}
          {props.nodeType === "Domain" && (
            <div class="grid grid-cols-[1fr_auto] gap-2">
              <For each={props.source()}>
                {(nodeId) => <DomainNode nodeId={nodeId} />}
              </For>
            </div>
          )}
          {props.nodeType === "SearchTerm" && (
            <div class="grid grid-cols-[1fr_auto] gap-2">
              <For each={props.source()}>
                {(nodeId) => <SearchTermNode nodeId={nodeId} />}
              </For>
            </div>
          )}
          {(props.nodeType === "Title" ||
            props.nodeType === "Heading" ||
            props.nodeType === "Paragraph") && (
            <div class="grid grid-cols-1 gap-2">
              <For each={props.source()}>
                {(nodeId) => <ContentNode nodeId={nodeId} />}
              </For>
            </div>
          )}
        </>
      ) : null}
    </>
  );
};

export default NodeGrid;
