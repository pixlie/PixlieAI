import { Component } from "solid-js";

const NodeEditor: Component = () => {
  // The Node editor is an HTML5 Canvas based editor for nodes and edges in the graph.

  // A blank canvas for drawing nodes and edges.
  return (
    <div>
      <canvas id="node-editor-canvas" width="800" height="600"></canvas>
    </div>
  );
};

export default NodeEditor;
