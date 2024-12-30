import { Component, JSX } from "solid-js";
import { IFormField } from "../../utils/types";
// import { ContentType } from "../../api_types/ContentType";
import { useUIClasses } from "../../stores/UIClasses";

const TextInput: Component<IFormField> = (props) => {
  const [_, { getColors }] = useUIClasses();
  // if (props.contentType !== ("Text" as ContentType)) {
  //   return null;
  // }

  let inputType: string = "text";
  // if (!!props.contentSpec.textType) {
  //   const textType = props.contentSpec.textType;
  //   if (textType === "Email") {
  //     inputType = "email";
  //   } else if (textType === "Password") {
  //     inputType = "password";
  //   }
  // }

  const handleChange: JSX.ChangeEventHandler<HTMLInputElement, Event> = (
    event,
  ) => {
    if (!!props.onChange) {
      props.onChange(props.name, event.currentTarget.value);
    }
  };

  // const LinkOptions: Component = () => {
  //   return (
  //     <>
  //       {props.value !== undefined && typeof props.value === "string" ? (
  //         <div class="font-thin">
  //           Click to{" "}
  //           <a
  //             href={props.value}
  //             class="font-medium text-blue-500 underline"
  //             rel="noreferrer"
  //             target="_blank"
  //           >
  //             open URL in default browser
  //           </a>{" "}
  //           or right click link to copy
  //         </div>
  //       ) : null}
  //     </>
  //   );
  // };

  return (
    <>
      {!!props.label && (
        <label
          class={`block text-sm font-medium leading-6 mb-1 ${getColors()["input.label"]}`}
        >
          {props.label}
        </label>
      )}
      <input
        name={props.name}
        type={inputType}
        required={props.isRequired || undefined}
        class={`block w-full rounded-md px-2 py-1.5 border font-content ${getColors()["input"]}`}
        placeholder={props.placeholder || undefined}
        value={props.value !== undefined ? props.value : ""}
        onChange={handleChange}
        onFocus={props.onFocus}
        disabled={!props.isEditable}
      />

      {/* {props.contentType === "Text" &&
      "textType" in props.contentSpec &&
      props.contentSpec.textType === "Link" ? (
        <LinkOptions />
      ) : null} */}
    </>
  );
};

export default TextInput;
