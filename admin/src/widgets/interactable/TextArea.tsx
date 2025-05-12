import { Component, JSX } from "solid-js";
import { useUIClasses } from "../../stores/UIClasses";
import { ITextAreaFormField } from "../../utils/types";

const TextArea: Component<ITextAreaFormField> = (props) => {
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
      rows={props.rows}
    />
  );
};

export default TextArea;
