import { Component, JSX } from "solid-js";
import ToolTip from "../navigation/ToolTip";

interface IconButtonProps {
  onClick: () => void;
  icon: JSX.Element;
  name: string;
  disabled?: boolean;
}

const IconButton: Component<IconButtonProps> = (props) => (
  <ToolTip text={props.name}>
    <button
      onClick={props.onClick}
      class="rounded-full p-2 hover:bg-slate-200"
      disabled={props.disabled || false}
    >
      {props.icon}
    </button>
  </ToolTip>
);

export default IconButton;
