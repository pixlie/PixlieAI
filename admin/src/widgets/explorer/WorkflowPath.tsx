import {
  Component,
  createEffect,
  createMemo,
  createSignal,
  onMount,
} from "solid-js";
import { useExplorer } from "../../stores/explorer.tsx";

interface IWorkflowPathProps {
  projectId: string;
  label: string;
  source: string;
  target: string;
}

const WorkflowPath: Component<IWorkflowPathProps> = (props) => {
  const [explorer] = useExplorer();
  const [path, setPath] = createSignal<string | undefined>(undefined);
  const [arrowPolygonPoints, setArrowPolygonPoints] = createSignal<
    string | undefined
  >(undefined);
  const [sourceXY, setSourceXY] = createSignal<
    { x: number; y: number } | undefined
  >(undefined);
  const [targetXY, setTargetXY] = createSignal<
    { x: number; y: number } | undefined
  >(undefined);
  const [sourceLayerBoundaries, setSourceLayerBoundaries] = createSignal<
    { leftX: number; rightX: number } | undefined
  >(undefined);
  const [targetLayerBoundaries, setTargetLayerBoundaries] = createSignal<
    { leftX: number; rightX: number } | undefined
  >(undefined);

  const getSourceXY = createMemo(() => {
    const project = explorer.projects[props.projectId];
    const elements = project.workflowElements;
    const sourceElement = elements[props.source];
    const sourceDom = sourceElement.state.dom;
    const sourceRel = sourceElement.state.relative;
    if (!sourceDom || !sourceRel) {
      return undefined;
    }
    const x = sourceRel.position.x + sourceDom.width;
    const y = sourceRel.position.y + sourceDom.height / 2;
    setSourceXY({ x, y });
  });

  const getTargetXY = createMemo(() => {
    const project = explorer.projects[props.projectId];
    const elements = project.workflowElements;
    const targetElement = elements[props.target];
    const targetDom = targetElement.state.dom;
    const targetRel = targetElement.state.relative;
    if (!targetDom || !targetRel) {
      return undefined;
    }
    const x = targetRel.position.x;
    const y = targetRel.position.y + targetDom.height / 2;
    setTargetXY({ x, y });
  });
  const getSourceLayerBoundaries = createMemo(() => {
    const project = explorer.projects[props.projectId];
    const elements = project.workflowElements;
    const sourceElement = elements[props.source];
    setSourceLayerBoundaries(
      project.layers[sourceElement.state.layer]?.boundaries,
    );
  });
  const getTargetLayerBoundaries = createMemo(() => {
    const project = explorer.projects[props.projectId];
    const elements = project.workflowElements;
    const targetElement = elements[props.target];
    setTargetLayerBoundaries(
      project.layers[targetElement.state.layer]?.boundaries,
    );
  });

  const updatePaths = createMemo(() => {
    getSourceXY();
    getTargetXY();
    getSourceLayerBoundaries();
    getTargetLayerBoundaries();
    if (
      !sourceXY() ||
      !targetXY() ||
      !sourceLayerBoundaries() ||
      !targetLayerBoundaries()
    ) {
      setPath(undefined);
    } else {
      setPath(
        `M ${sourceXY()!.x} ${sourceXY()!.y} C ${(sourceLayerBoundaries()!.rightX + targetLayerBoundaries()!.leftX) / 2} ${sourceXY()!.y} ${(sourceLayerBoundaries()!.rightX + targetLayerBoundaries()!.leftX) / 2} ${targetXY()!.y} ${targetXY()!.x} ${targetXY()!.y}`,
      );
    }
    if (!targetXY()) {
      setArrowPolygonPoints(undefined);
    } else {
      setArrowPolygonPoints(
        `${targetXY()!.x} ${targetXY()!.y} ${targetXY()!.x} ${targetXY()!.y} ${targetXY()!.x - 6} ${targetXY()!.y - 3} ${targetXY()!.x - 6} ${targetXY()!.y + 3}`,
      );
    }
  });

  onMount(() => {
    updatePaths();
  });

  createEffect(() => {
    updatePaths();
  });

  return (
    <>
      {path() && <path d={path()} stroke="gray" fill="none" />}
      {sourceLayerBoundaries() &&
        targetLayerBoundaries() &&
        sourceXY() &&
        targetXY() && (
          <text
            x={
              (sourceLayerBoundaries()!.rightX +
                targetLayerBoundaries()!.leftX) /
              2
            }
            y={(sourceXY()!.y + targetXY()!.y) / 2}
            font-size="9"
            fill="gray"
            text-anchor="middle"
            dominant-baseline="middle"
            style={{
              "pointer-events": "none",
              "user-select": "none",
            }}
          >
            {props.label}
          </text>
        )}
      {sourceXY() && (
        <circle
          cx={sourceXY()!.x}
          cy={sourceXY()!.y}
          r="8"
          fill="lightgray"
          style={{
            "pointer-events": "none",
            "user-select": "none",
          }}
        />
      )}
      {arrowPolygonPoints() && (
        <polygon
          points={arrowPolygonPoints()}
          fill="gray"
          style={{
            "pointer-events": "none",
            "user-select": "none",
          }}
        />
      )}
    </>
  );
};

export default WorkflowPath;
