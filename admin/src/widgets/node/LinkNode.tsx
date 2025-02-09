import { Link } from "../../api_types/Link.ts";
import { Component } from "solid-js";
import { useUIClasses } from "../../stores/UIClasses.tsx";

interface ILinkProps extends Link {}

const LinkNode: Component<ILinkProps> = (props) => {
  const [_, { getColors }] = useUIClasses();

  return (
    <div class="grid grid-cols-2 items-center">
      <a href={props.url} class={getColors().link}>
        {props.url}
      </a>
      <span>{props.is_fetched ? "Fetched" : "Not Fetched"}</span>
    </div>
  );
};

export default LinkNode;
