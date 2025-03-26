import { Component } from "solid-js/types/server/rendering.js";

const Toggle: Component = () => {
  return (
    <button
      type="button"
      class="relative inline-flex h-6 w-11 shrink-0 cursor-pointer rounded-full border-2 border-transparent bg-gray-200 transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-indigo-600 focus:ring-offset-2"
      role="switch"
      aria-checked="false"
    >
      <span class="sr-only">Use setting</span>
      {/* Enabled: "translate-x-5", Not Enabled: "translate-x-0" */}
      <span
        aria-hidden="true"
        class="pointer-events-none inline-block size-5 translate-x-0 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out"
      ></span>
    </button>
  );
};

export default Toggle;
