import { Component, JSX } from "solid-js";
import { useUIClasses } from "../../stores/UIClasses";

interface IPropTypes {
  size: 1 | 2 | 3 | 4 | 5 | 6;
  children: JSX.Element | string;
}

const Heading: Component<IPropTypes> = (props): JSX.Element => {
  // We need this function in order to tell Tailwind CSS exact and full size classes, so it will include all of these.
  // Otherwise, Tailwind will not include them in the final CSS file.
  const [_, { getColors }] = useUIClasses();

  const getSizeClass = (size: number) => {
    switch (size) {
      case 5:
        return "text-2xl font-light leading-tight";
      case 4:
        return "text-3xl font-light leading-tight";
      case 3:
        return "text-4xl font-light leading-snug";
      case 2:
        return "text-5xl font-light leading-snug";
      case 1:
        return "text-6xl font-light leading-snug";
      case 6:
      default:
        return "text-xl font-light leading-tight";
    }
  };
  const headingClasses = `${getSizeClass(
    props.size,
  )} block select-none cursor-default`;

  return (
    <span class={headingClasses} style={{ color: getColors().heading }}>
      {props.children}
    </span>
  );
};

export default Heading;
