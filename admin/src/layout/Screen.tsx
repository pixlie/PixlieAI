import { Component, JSX } from "solid-js";

export interface ScreenProps {
  title: string;
  children?: JSX.Element;
}

const Screen: Component<ScreenProps> = (props) => {
  return (
    <div class="h-full w-full flex justify-center items-center">
      {props.title && <h1 class="text-5xl font-medium">{props.title}</h1>}
      {props.children && props.children}
    </div>
  );
};

export default Screen;
