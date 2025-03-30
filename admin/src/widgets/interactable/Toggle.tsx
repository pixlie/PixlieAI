import { Component } from "solid-js/types/server/rendering.js";
import { IBooleanFormField } from "../../utils/types.tsx";
import { createMemo, JSX } from "solid-js";

const Toggle: Component<IBooleanFormField> = (props) => {
  const handleChange: JSX.ChangeEventHandler<HTMLButtonElement, Event> = () => {
    if (!!props.onChange) {
      props.onChange(props.name, !props.value);
    }
  };

  // Enabled: "bg-indigo-600", Not Enabled: "bg-gray-200"
  const getButtonClasses = createMemo<string>(
    () =>
      `relative inline-flex h-6 w-11 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-indigo-600 focus:ring-offset-2 ${
        props.value ? "bg-indigo-600" : "bg-gray-200"
      }`,
  );

  // Enabled: "translate-x-5", Not Enabled: "translate-x-0"
  let getToggleClasses = createMemo<string>(
    () =>
      `pointer-events-none inline-block size-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out ${
        !!props.value ? "translate-x-5" : "translate-x-0"
      }`,
  );

  return (
    <button
      type="button"
      class={getButtonClasses()}
      role="switch"
      aria-checked={!!props.value ? "true" : "false"}
      onClick={handleChange}
      id={props.name}
    >
      <span class="sr-only">Use setting</span>
      <span aria-hidden="true" class={getToggleClasses()}></span>
    </button>
  );
};

export default Toggle;
