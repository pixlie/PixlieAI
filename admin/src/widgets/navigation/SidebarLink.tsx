import { Component } from "solid-js";
import { useUIClasses } from "../../stores/UIClasses";
import { IRoute } from "../../routes/RouteList.tsx";

const SidebarLink: Component<IRoute> = (props) => {
  const [_, { getColors }] = useUIClasses();

  return (
    <>
      {props.href ? (
        <a
          class={
            "block rounded-md px-2 py-1.5 text-sm " +
            getColors()["sideBar.link"] +
            (props.isActive ? " " + getColors()["sideBar.link.active"] : "")
          }
          href={props.href}
        >
          {props.label}
        </a>
      ) : (
        <div
          class={
            "block rounded-md px-2 py-1.5 " +
            getColors()["sideBar.label"] +
            (props.isActive
              ? " font-bold " + getColors()["sideBar.label.active"]
              : "")
          }
        >
          {props.label}
        </div>
      )}
    </>
  );
};

export default SidebarLink;
