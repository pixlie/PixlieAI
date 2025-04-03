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
  return (
    <div class="inline-block group/actions-wrapper relative">
      <div class="gap-0 flex justify-center absolute top-1/2 -translate-y-1 opacity-0 transition duration-200 group-hover/actions-wrapper:opacity-100 group-hover/actions-wrapper:top-full group-hover/actions-wrapper:translate-y-0 w-full">
        <For each={props.actions}>
          {(action) => {
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
      {props.children}
    </div>
  );
};

export default ActionsWrapper;
