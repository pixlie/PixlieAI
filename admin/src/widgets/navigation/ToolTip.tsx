import { Component, JSX } from "solid-js";

interface ToolTipProps {
  children: JSX.Element;
  text: string;
}

const ToolTip: Component<ToolTipProps> = (props) => {
  return (
    <div class="relative group inline-block">
      <div>{props.children}</div>
      <div class="pointer-events-none absolute top-full left-1/2 -translate-x-1/2 mt-0.5 px-2 py-1.5 text-sm font-medium text-white bg-gray-900 rounded-md whitespace-nowrap opacity-0 translate-y-1 scale-95 transition duration-150 ease-out group-hover:opacity-100 group-hover:translate-y-0 group-hover:scale-100 z-10">
        {props.text}
      </div>
    </div>
  );
};

export default ToolTip;
