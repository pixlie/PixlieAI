import { Component } from "solid-js";
import { useUIClasses } from "../../stores/UIClasses";
import GetIcon from "../../utils/Icons";

interface IPropTypes {
  label: string;
  icon: string;
  href: string;
  isActive?: boolean;
}

const SidebarLink: Component<IPropTypes> = (props) => {
  const [_, { getColors }] = useUIClasses();

  // let classes =
  //   "group flex gap-x-3 rounded-md p-2 text-sm/6 font-semibold " +
  //   getColors()["sideBar.link"];
  let classes =
    "group flex items-center gap-x-3 rounded-md px-2 py-4 text-sm/6 font-semibold " +
    getColors()["sideBar.link"];

  return (
    <li>
      <a class={classes} href={props.href}>
        <GetIcon iconName={props.icon} />
        <span class="text-sm font-semibold">{props.label}</span>
      </a>
    </li>
  );
};

export default SidebarLink;
