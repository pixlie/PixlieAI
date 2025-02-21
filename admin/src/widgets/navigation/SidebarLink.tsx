import { Component } from "solid-js";
import { useUIClasses } from "../../stores/UIClasses";
import { IRoute } from "../../routes/RouteList.tsx";
import { A } from "@solidjs/router";

const SidebarLink: Component<IRoute> = (props) => {
  const [_, { getColors }] = useUIClasses();

  return (
    <>
      {props.href ? (
        <A
          class={
            "block rounded-md px-3 py-2 text-md w-full " +
            getColors()["sideBar.link"] +
            (props.isActive ? " font-semibold" : "")
          }
          href={props.href}
        >
          {props.label}
        </A>
      ) : (
        <div
          class={
            "block rounded-md px-3 py-2 text-md w-full " +
            getColors()["sideBar.label"] +
            (props.isActive ? " font-semibold" : "")
          }
        >
          {props.label}
        </div>
      )}
    </>
  );
};

export default SidebarLink;
