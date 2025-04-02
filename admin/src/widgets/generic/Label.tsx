import { Component } from "solid-js";
import { ILabel } from "../../utils/types";
import { useUIClasses } from "../../stores/UIClasses";
import ToolTip from "../navigation/ToolTip";

const Label: Component<ILabel> = (props) => {
  const [_, { getColors }] = useUIClasses();
  const labelRef = (
    <span
      title={props.title}
      class={`px-2.5 py-0.5 rounded-2xl cursor-pointer text-sm text-center ${getColors()[props.color]} ${props.class ? props.class : ""}`}
      onClick={props.onClick}
    >
      {props.children}
    </span>
  );
  if (props.tooltip) {
    return (
      <ToolTip text={props.tooltip} textSize="xs">
        {labelRef}
      </ToolTip>
    );
  }
  return labelRef;
};

export default Label;
