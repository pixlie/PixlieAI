import { Component, createMemo } from "solid-js";
import { useUIClasses } from "../../stores/UIClasses";
import { useEngine } from "../../stores/engine";
import { useParams } from "@solidjs/router";
import { IEngine } from "../../utils/types";
import { NodeLabel } from "../../api_types/NodeLabel";

interface IDomainNodeProps {
  nodeId: number;
}

const DomainNode: Component<IDomainNodeProps> = (props) => {
  const [engine] = useEngine();
  const params = useParams();
  const [_, { getColors }] = useUIClasses();

  const getProject = createMemo<IEngine | undefined>(() => {
    if (!!params.projectId && params.projectId in engine.projects) {
      return engine.projects[params.projectId];
    }
    return undefined;
  });

  const getDomain = createMemo<string | undefined>(() => {
    if (getProject() && props.nodeId in getProject()!.nodes) {
      let node = getProject()!.nodes[props.nodeId];
      if (
        node.labels.includes("Domain" as NodeLabel) &&
        node.payload.type === "Text"
      ) {
        return node.payload.data as string;
      }
      return undefined;
    }
    return undefined;
  });

  const getRobotsTxtStatus = createMemo<string>(() => {
    if (getDomain() && engine.projects[params.projectId].edges) {
      let domain = getProject()!.nodes[props.nodeId];
      if (domain.id in engine.projects[params.projectId].edges) {
        let edges = engine.projects[params.projectId].edges[domain.id];
        let ownerOfNodeIds = edges
          .filter((edge) => edge[1] === "OwnerOf")
          .map((edge) => edge[0]);

        let robotsTxtNode = ownerOfNodeIds
          .map((nodeId) => engine.projects[params.projectId].nodes[nodeId])
          .find((node) => node.labels.includes("RobotsTxt"));

        if (robotsTxtNode) {
          console.log(robotsTxtNode.flags.join(", "));
          if (
            robotsTxtNode.flags.filter((flag) => flag === "IsRequesting")
              .length > 0
          ) {
            return "Requesting robots.txt";
          } else if (
            robotsTxtNode.flags.filter((flag) => flag === "IsBlocked").length >
            0
          ) {
            return "Found robots.txt";
          }
        }
      }
    }
    return "";
  });

  return (
    <>
      {!!getDomain() ? (
        <>
          <a
            class={getColors().link}
            href={`https://${getDomain()!}`}
            target="_blank"
            rel="noreferrer"
          >
            {getDomain()!}
          </a>

          <span>{getRobotsTxtStatus()}</span>
        </>
      ) : null}
    </>
  );
};

export default DomainNode;
