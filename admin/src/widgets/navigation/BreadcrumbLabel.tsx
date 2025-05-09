import { Component } from "solid-js";
import { useUIClasses } from "../../stores/UIClasses";
import ChevronRightIcon from "../../assets/icons/tabler-chevron-right.svg";

interface IPropTypes {
  label?: string;
  href?: string;
  isLast?: boolean;
}

const BreadcrumbLabel: Component<IPropTypes> = (props) => {
  const [_, { getColors }] = useUIClasses();

  return (
    <>
      {props.label && (
        <p
          class={
            "text-md cursor-default leading-none  " + getColors()["textLight"]
          }
        >
          {props.label}
        </p>
      )}
      {!props.isLast && (
        <div
          class={
            "w-2.5 h-2.5 flex items-center justify-center mr-2 " +
            getColors()["textMuted"]
          }
        >
          <ChevronRightIcon />
        </div>
      )}
    </>
  );
};

export default BreadcrumbLabel;
