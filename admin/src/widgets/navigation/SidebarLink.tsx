import { Component } from "solid-js";
import { useUIClasses } from "../../stores/UIClasses";

interface IPropTypes {
  label: string;
  href: string;
  isActive?: boolean;
}

const SidebarLink: Component<IPropTypes> = (props) => {
  const [_, { getColors }] = useUIClasses();

  return (
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
  );
};

export default SidebarLink;
