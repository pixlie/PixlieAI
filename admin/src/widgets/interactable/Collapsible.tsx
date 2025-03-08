import { Component, JSX, createEffect, createSignal } from "solid-js";
import { createElementSize } from "@solid-primitives/resize-observer";

interface IPropTypes {
  children: JSX.Element;
  maxHeight?: number;
  expandText?: string;
  collapseText?: string;
  onClickExpand?: (e: MouseEvent) => void;
  onClickCollapse?: (e: MouseEvent) => void;
  isCollapsed?: boolean;
}

const Collapsible: Component<IPropTypes> = (props) => {
  const [isCollapsed, setIsCollapsed] = createSignal<boolean>(
    props.isCollapsed === undefined
      ? true
      : props.isCollapsed,
  );
  const [maxHeight] = createSignal<number>(props.maxHeight || 150);
  const [isOverflowing, setIsOverflowing] = createSignal<boolean>(false);
  const [contentRef, setContentRef] = createSignal<HTMLElement>();

  const contentElementSize = createElementSize(contentRef);
  createEffect(() => {
    if (contentElementSize.height) {
      setIsOverflowing(contentElementSize.height > maxHeight());
    }
  });

  const handleToggle = (e: MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    if (isCollapsed()) {
      setIsCollapsed(false);
      if (props.onClickExpand) {
        props.onClickExpand(e);
      }
    }
    else {
      setIsCollapsed(true);
      if (props.onClickCollapse) {
        props.onClickCollapse(e);
      }
    }
  };
  
  return (
    <div
      class="relative transition-all overflow-y-clip"
      classList={{
        "max-h-[150px]": isOverflowing() && isCollapsed(),
        "max-h-none": !isOverflowing() || !isCollapsed(),
      }}
    >
      <div
        ref={setContentRef}
        classList={{
          "mb-0": isCollapsed(),
          "mb-10": !isCollapsed(),
        }}
      >
        {props.children}
      </div>
      {isOverflowing() && (
        <div
          class={`absolute bottom-0 left-0 right-0 bg-gradient-to-t from-white via-white to-transparent transition-all flex flex-col place-items-center h-10`}
        >
          <a
            href="javasript:void(0)"
            class={`absolute bottom-0 text-sm bg-gradient-to-t from-transparent via-white to-white rounded-t-md px-2 py-1 h-8 inline-block drop-shadow-[0_-5px_5px_rgba(0,0,0,0.25)]`}
            onClick={handleToggle}
          >
            {isCollapsed()
              ? `↓ ${props.expandText || "Expand"} ↓`
              : `↑ ${props.collapseText || "Collapse"} ↑`}
          </a>
        </div>
      )}
    </div>
  );
};

export default Collapsible;
