import { Component } from "solid-js";
import { Project } from "../../api_types/Project";
import { A } from "@solidjs/router";

interface ProjectProps extends Project {}

const ProjectListItem: Component<ProjectProps> = (props) => {
  return (
    <div class="rounded bg-white p-6 relative hover:drop-shadow-md">
      <span class="text-base font-semibold text-gray-900">
        <A href={`/p/${props.uuid}/workflow`}>
          {/* Extend touch target to entire panel */}
          <span class="absolute inset-0" aria-hidden="true"></span>
          {props.name}
        </A>
      </span>
      {!!props.description && (
        <p class="mt-2 text-sm text-gray-500">{props.description}</p>
      )}
    </div>
  );
};

export default ProjectListItem;
