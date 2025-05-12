import { Component, JSX } from "solid-js";
import { useUIClasses } from "../../stores/UIClasses";
import { ThemableItem } from "../../utils/uIClasses/types";
import ToolTip from "../navigation/ToolTip";

interface IconButtonProps {
  onClick: () => void;
  icon: JSX.Element;
  name: string;
  disabled?: boolean;
  position?: "top" | "bottom";
  color?: ThemableItem;
}

const IconButton: Component<IconButtonProps> = (props) => {
  const [_, { getColors }] = useUIClasses();
  return (
    <ToolTip text={props.name} position={props.position || "bottom"}>
      <button
        onClick={props.onClick}
        class="rounded-full p-2"
        classList={{
          [getColors()[props.color!]]: !!props.color,
          "hover:bg-slate-200 hover:text-slate-800": !props.color,
          "opacity-70 hover:opacity-100": !!props.color && !props.disabled,
          "opacity-50": props.disabled,
        }}
        disabled={props.disabled || false}
      >
        {props.icon}
      </button>
    </ToolTip>
  );
};

export default IconButton;
