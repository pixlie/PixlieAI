import { Component } from "solid-js";
import Heading from "../../widgets/typography/Heading";

const LocalPythonEnv: Component = () => {
  // We need a local Python virtual environment. We are our API if it can detect system Python and venv.
  return (
    <>
      <Heading size={3}>Python</Heading>
    </>
  );
};

export default LocalPythonEnv;
