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
        class={`inline-block size-4`}
        classList={{
          "cursor-pointer hover:opacity-100 hover:scale-125 opacity-90":
            !!props.onClick,
          "opacity-60": !props.onClick,
          [getColors()[props.color || "textSoft"]]: !!props.onClick,
          [getColors()[props.color || "textMuted"]]: !props.onClick,
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
  return (
    <div class="inline-flex group/actions-wrapper flex-col gap-0">
      <span class="invisible group-hover/actions-wrapper:visible px-0.5 gap-1 flex items-end justify-center">
        <For each={props.actions}>
          {(action) => {
            const actionRef = (
              <ActionsWrapperAction id={props.id} {...action} />
            );
            if (!!action.tooltip) {
              return (
                <ToolTip text={action.tooltip} textSize="xs" position="top">
                  {actionRef}
                </ToolTip>
              );
            }
            return actionRef;
          }}
        </For>
      </span>
      {props.children}
    </div>
  );
};

export default ActionsWrapper;
