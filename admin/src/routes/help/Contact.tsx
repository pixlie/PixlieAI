import { RouteSectionProps } from "@solidjs/router";
import { Component } from "solid-js";

const Contact: Component<RouteSectionProps> = () => (
  <div class="h-dvh w-dvw">
    <p class="text-gray-700">
      {`Need help? Email `}
      <span>
        <a
          href="mailto:team@pixlie.com"
          class="text-blue-700 underline font-medium"
          target="_blank"
          rel="noreferrer"
        >
          team@pixlie.com
        </a>
      </span>
    </p>
  </div>
);

export default Contact;
