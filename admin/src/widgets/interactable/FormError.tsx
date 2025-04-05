import { Accessor, Component } from "solid-js";
import { useUIClasses } from "../../stores/UIClasses.tsx";

interface IPropTypes {
  name: string;
  errors: Accessor<{ [key: string]: string }>;
}

const FormError: Component<IPropTypes> = (props) => {
  const [_, { getColors }] = useUIClasses();

  return (
    <>
      {!!props.errors()[props.name] && (
        <div class={`text-sm ${getColors()["formError"]}`}>
          {props.errors()[props.name]}
        </div>
      )}
    </>
  );
};

export default FormError;
