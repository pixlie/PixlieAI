import { Component } from "solid-js";
import { useUIClasses } from "../../stores/UIClasses";
import { IRoute } from "../../routes/RouteList.tsx";
import { A } from "@solidjs/router";
import ChevronDownIcon from "../../assets/icons/tabler-chevron-down.svg";

const SidebarLink: Component<IRoute> = (props) => {
  const [_, { getColors }] = useUIClasses();

  return (
    <>
      {props.href ? (
        <div
          class={
            `transition-colors duration-150 text-md ${getColors()["text"]} hover:${getColors()["textDark"]}` +
            (props.isActive
              ? ` font-medium bg-blue-100 hover:bg-blue-100 ${getColors()["textDark"]}`
              : " hover:bg-slate-200")
          }
        >
          <A
            class={"block py-3 mx-8 " + (props.isChild ? "pl-4" : "")}
            href={props.href}
          >
            {props.label}
          </A>
        </div>
      ) : (
        <div class="flex items-center gap-2 mx-8 py-3 ">
          <p class={`text-md cursor-default ${getColors()["text"]}`}>
            {props.label}
          </p>
          <div
            class={
              "h-5 w-5 flex justify-center items-center " +
              getColors()["textMuted"]
            }
          >
            <ChevronDownIcon />
          </div>
        </div>
      )}
    </>
  );
};

export default SidebarLink;
