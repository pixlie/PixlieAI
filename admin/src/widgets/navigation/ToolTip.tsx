import { Component, JSX } from "solid-js";

interface ToolTipProps {
  children: JSX.Element;
  text: string;
  textSize?: "xs" | "sm" | "md" | "lg" | "xl";
  position?: "top" | "bottom";
}

const ToolTip: Component<ToolTipProps> = (props) => {
  if (!props.textSize) {
    props.textSize = "sm";
  }
  if (!props.position) {
    props.position = "bottom";
  }
  return (
    <div class="relative group/tooltip">
      <div class="m-0 p-0">{props.children}</div>
      <div
        class="absolute hidden group-hover/tooltip:block left-1/2 transform -translate-x-1/2 px-2 py-1 text-white bg-gray-900 rounded-lg whitespace-nowrap z-50 drop-shadow-[0_0_5px_white]"
        classList={{
          "text-xs": props.textSize === "xs",
          "text-sm": props.textSize === "sm",
          "text-md": props.textSize === "md",
          "text-lg": props.textSize === "lg",
          "text-xl": props.textSize === "xl",
          "bottom-full mb-3": props.position === "top",
          "top-full mt-3": props.position === "bottom",
        }}
      >
        {props.position === "top" && (
          <div class="absolute top-full left-1/2 transform -translate-x-1/2 mb-1">
            <div class="border-solid border-t-gray-900 border-b-0 border-x-transparent border-x-8 border-t-8" />
          </div>
        )}
        {props.text}
        {props.position === "bottom" && (
          <div class="absolute bottom-full left-1/2 transform -translate-x-1/2 mt-1">
            <div class="border-solid border-b-gray-900 border-b-8 border-x-transparent border-x-8 border-t-0" />
          </div>
        )}
      </div>
    </div>
  );
};

export default ToolTip;
