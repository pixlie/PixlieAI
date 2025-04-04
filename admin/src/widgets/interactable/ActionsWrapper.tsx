import { Component, For } from "solid-js";
import { IActionsWrapper, IActionsWrapperAction } from "../../utils/types.tsx";
import { useUIClasses } from "../../stores/UIClasses.tsx";
import ToolTip from "../navigation/ToolTip.tsx";

interface IWrapperAction extends IActionsWrapperAction {
  id: string;
}

const ActionsWrapperAction: Component<IWrapperAction> = (props) => {
  const [_, { getColors }] = useUIClasses();
  return (
    props.render && (
      <span
        class={`inline-block size-4 transform transition duration-100`}
        classList={{
          "cursor-pointer scale-105 hover:opacity-100 hover:scale-125 opacity-90":
            !!props.onClick,
          "opacity-60": !props.onClick,
          [getColors()[props.color || "textSoft"]]: !!props.onClick,
          [getColors()["textMuted"]]: !props.onClick,
        }}
        onClick={() => {
          if (props.onClick) {
            props.onClick(props.id);
          }
        }}
        title={!!props.icon ? props.text : ""}
      >
        {props.icon || props.text}
      </span>
    )
  );
};

const ActionsWrapper: Component<IActionsWrapper> = (props) => {
  if (props.flipActionsAndTooltip === undefined) {
    props.flipActionsAndTooltip = false;
  }
  return (
    <div class="inline-block group/actions-wrapper relative">
      {/* <div class="w-[200%] min-w-min justify-center items-center"> */}
      <div
        class="gap-0.5 flex justify-center items-center place-items-start px-3 py-1.5 leading-4 absolute left-1/2 -z-50 transform -translate-x-1/2 delay-0 bg-white rounded-lg shadow-inner drop-shadow-md opacity-0 transition duration-100 group-hover/actions-wrapper:opacity-100 group-hover/actions-wrapper:z-50 group-hover/actions-wrapper:delay-500"
        classList={{
          "top-full -translate-y-1 group-hover/actions-wrapper:-translate-y-[1px]":
            !props.flipActionsAndTooltip,
          "bottom-full translate-y-1 group-hover/actions-wrapper:translate-y-[1px]":
            props.flipActionsAndTooltip,
        }}
      >
        <For each={props.actions}>
          {(action) => {
            if (!action.render) {
              return null;
            }
            const actionRef = (
              <ActionsWrapperAction id={props.id} {...action} />
            );
            if (!!action.tooltip) {
              return (
                <ToolTip text={action.tooltip} textSize="xs" position="bottom">
                  {actionRef}
                </ToolTip>
              );
            }
            return actionRef;
          }}
        </For>
      </div>
      <span class="z-0">{props.children}</span>
    </div>
    // </div>
  );
};

export default ActionsWrapper;
