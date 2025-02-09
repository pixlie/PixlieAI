import { Component } from "solid-js";
import { useUIClasses } from "../../stores/UIClasses.tsx";
import { Domain } from "../../api_types/Domain.ts";

interface ILinkProps extends Domain {}

const DomainNode: Component<ILinkProps> = (props) => {
  const [_, { getColors }] = useUIClasses();

  return (
    <div class="grid grid-cols-2 items-center">
      <a href={`https://${props.name}`} class={getColors().link}>
        {props.name}
      </a>
      <span>{props.is_allowed_to_crawl ? "Can crawl" : "Cannot crawl"}</span>
    </div>
  );
};

export default DomainNode;
