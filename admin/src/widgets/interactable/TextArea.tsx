import { Component, JSX } from "solid-js";
import { ITextFormField } from "../../utils/types";
import { useUIClasses } from "../../stores/UIClasses";

const TextArea: Component<ITextFormField> = (props) => {
  const [_, { getColors }] = useUIClasses();

  const handleChange: JSX.ChangeEventHandler<HTMLTextAreaElement, Event> = (
    event,
  ) => {
    if (!!props.onChange) {
      props.onChange(props.name, event.currentTarget.value);
    }
  };

  return (
    <textarea
      id={props.id}
      name={props.name}
      required={props.isRequired !== null ? props.isRequired : false}
      class={`block w-full rounded-md px-4 py-2.5 border outline-none font-content m-0  ${getColors()["input"]}`}
      placeholder={props.placeholder !== null ? props.placeholder : ""}
      value={props.value || ""}
      onChange={handleChange}
      disabled={!props.isEditable}
    />
  );
};

export default TextArea;
