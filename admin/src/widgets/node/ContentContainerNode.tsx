import { Component, createMemo, For } from "solid-js";
import { useEngine } from "../../stores/engine";
import { useParams } from "@solidjs/router";
import { INodeItemDisplayProps } from "../../utils/types";
import { APINodeItem } from "../../api_types/APINodeItem";
import { NodeLabel } from "../../api_types/NodeLabel";
import ContentNode from "./ContentNode";

const ContentContainerNode: Component<INodeItemDisplayProps> = (props) => {
  const [engine] = useEngine();
  const params = useParams();

  const getProject = createMemo(() => {
    if (!!params.projectId && params.projectId in engine.projects) {
      return engine.projects[params.projectId];
    }
    return undefined;
  });

  const getContentNodes = createMemo<Array<APINodeItem>>(() => {
    if (
      !!getProject() &&
      props.nodeId in getProject()!.nodes &&
      props.nodeId in getProject()!.edges
    ) {
      let nodes: APINodeItem[] = [];

      // Get all nodes connected with the ContentOf edge
      getProject()!.edges[props.nodeId].edges.forEach((edge) => {
        if (edge[1] === "ChildOf") {
          let contentNode = getProject()!.nodes[edge[0]];
          if (
            contentNode.labels.includes("Partial" as NodeLabel) &&
            contentNode.payload.type === "Text" &&
            contentNode.payload.data.length > 0
          ) {
            nodes.push(contentNode);
          }
        }
      });
      // Order nodes by their ID
      nodes.sort((a, b) => a.id - b.id);

      return nodes;
    }
    return [];
  });

  return (
    <>
      {getContentNodes().length > 0 ? (
        <div class="border border-gray-300 shadow-md rounded p-6">
          <For each={getContentNodes()}>
            {(node) => <ContentNode nodeId={node.id} />}
          </For>
        </div>
      ) : null}
    </>
  );
};

export default ContentContainerNode;
