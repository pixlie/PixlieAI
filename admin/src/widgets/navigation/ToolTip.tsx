import { Component, JSX } from "solid-js";

interface ToolTipProps {
  children: JSX.Element;
  text: string;
}

const ToolTip: Component<ToolTipProps> = (props) => {
  return (
    <div class="relative group">
      <div>{props.children}</div>
      <div class="absolute hidden group-hover:block top-full left-1/2 transform -translate-x-1/2 mt-3 px-2 py-1 text-sm text-white bg-gray-900 rounded-lg whitespace-nowrap">
        {props.text}
        <div class="absolute bottom-full left-1/2 transform -translate-x-1/2 mt-1">
          <div class="border-solid border-b-gray-900 border-b-8 border-x-transparent border-x-8 border-t-0" />
        </div>
      </div>
    </div>
  );
};

export default ToolTip;
