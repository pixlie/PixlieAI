import { JSX, JSXElement } from "solid-js";
import { APINodeEdges } from "../api_types/APINodeEdges.ts";
import { APINodeItem } from "../api_types/APINodeItem";
import { EdgeLabel } from "../api_types/EdgeLabel.ts";
import { NodeLabel } from "../api_types/NodeLabel.ts";
import { Project } from "../api_types/Project";
import { Settings } from "../api_types/Settings";
import { SettingsStatus } from "../api_types/SettingsStatus";
import { Workspace } from "../api_types/Workspace";
import { ThemableItem } from "./uIClasses/types";

interface IProviderPropTypes {
  children: JSX.Element;
}

interface IWorkspace {
  settings?: Settings;
  workspace?: Workspace;
  settingsStatus?: SettingsStatus;
  projects?: Array<Project>;

  isReady: boolean;
  isFetching: boolean;
}

interface INodeItem extends APINodeItem {
  isFetching: boolean;
}

interface IEngineNodes {
  [nodeId: number]: INodeItem;
}

interface IEngineEdges {
  [nodeId: number]: APINodeEdges;
}

interface IEngine {
  nodes: IEngineNodes;
  edges: IEngineEdges;
  nodesFetchedUpto: bigint;
  edgesFetchedUpto: bigint;
  isFetching: boolean;
}

interface IEngineStore {
  projects: { [projectId: string]: IEngine };
  sync: Array<string>;
}

interface IExplorerRootElement {
  domState: DOMRect | undefined;
}

interface IExplorerElementRelativePosition {
  x: number;
  y: number;
}
interface IExplorerElementRelativeSize {
  w: number;
  h: number;
}

interface IExplorerWorkflowElementState {
  position: IExplorerElementRelativePosition;
  size: IExplorerElementRelativeSize;
}

interface IExplorerWorkflowEdges {
  [label: string]: Array<string>;
}

interface IExplorerWorkflowElement {
  id: string;
  state: {
    dom: DOMRect | undefined;
    relative: IExplorerWorkflowElementState | undefined;
    layer: number;
  };
  labels: Array<NodeLabel>;
  edges: IExplorerWorkflowEdges;
  type: WorkflowElementType;
  nodeIds: number[];
}

interface IExplorerWorkflowNode {
  id: string;
  treeSize: number;
  children: IExplorerWorkflow;
}

type IExplorerWorkflow = IExplorerWorkflowNode[];

interface IExplorerNodes {
  [nodeId: string]: APINodeItem;
}
interface IExplorerEdges {
  [nodeId: string]: APINodeEdges;
}

interface IExplorerWorkflowElements {
  [key: string]: IExplorerWorkflowElement;
}

interface IExplorerWorkflowDisplayState {
  scale: number;
  size: {
    width: number;
    height: number;
  };
  translate: {
    x: number;
    y: number;
  };
}

interface IExplorerWorkflowLayer {
  height: number;
  width: number;
  boundaries: {
    topY: number;
    bottomY: number;
    leftX: number;
    rightX: number;
  };
}
type IExplorerWorkflowLayers = IExplorerWorkflowLayer[];

interface IExplorerProject {
  nodes: IExplorerNodes;
  edges: IExplorerEdges;
  siblingNodes: number[][];
  rootElement: IExplorerRootElement;
  displayState: IExplorerWorkflowDisplayState;
  workflow: IExplorerWorkflowNode[];
  layers: IExplorerWorkflowLayers;
  workflowElements: IExplorerWorkflowElements;
  loaded: boolean;
  ready: boolean;
}

interface IExplorerSettings {
  nodeLabelsOfInterest: NodeLabel[];
  configurableNodeLabels: NodeLabel[];
  edgeLabelsOfInterest: EdgeLabel[];
  horizontalSpacing: number;
  verticalSpacing: number;
  horizontalMargin: number;
  verticalMargin: number;
}

interface IExplorerStore {
  projects: { [projectId: string]: IExplorerProject };
  settings: IExplorerSettings;
}

interface INodeItemDisplayProps {
  nodeId: number;
  mode: "regular" | "preview";
  showFlags?: boolean;
  data?: Record<string, any>;
  nodeData?: Record<string, any>;
}

interface INodeListItemProps {
  nodeType?: string;
  source: () => Array<number>;
  mode: "regular" | "preview";
  data?: {
    data: { [key: string]: any };
    nodeData?: Record<string, any>;
  };
}

interface IFormField {
  id?: string;
  name: string;
  placeholder?: string | null;
  size?: "xs" | "sm" | "base" | "lg";
  displayBlock?: boolean;
  onFocus?: () => void;
  onKeyUp?: (event: KeyboardEvent) => void;
  isRequired?: boolean;
  isEditable?: boolean;
  autocomplete?: boolean;
}

interface ITextFormField extends IFormField {
  value?: string | number;
  onChange?: (name: string, value: string | number) => void;
}

interface IBooleanFormField extends IFormField {
  value?: boolean;
  onChange?: (name: string, value: boolean) => void;
}

interface IActionsWrapperAction {
  render: boolean;
  color?: ThemableItem;
  onClick?: (id: string) => void;
  text?: string;
  tooltip?: string;
  icon?: JSX.Element;
}

interface IActionsWrapper {
  id: string;
  children: string | JSX.Element | JSX.Element[];
  actions: IActionsWrapperAction[];
  tooltip?: string;
  flipActionsAndTooltip?: boolean;
  class?: JSX.HTMLAttributes<HTMLSpanElement>["class"];
  title?: JSX.HTMLAttributes<HTMLSpanElement>["title"];
  onClick?: (id: string) => void;
}

interface ILabel {
  children: string | JSXElement;
  color: ThemableItem;
  tooltip?: string;
  tooltipPosition?: "top" | "bottom";
  title?: JSX.HTMLAttributes<HTMLSpanElement>["title"];
  class?: JSX.HTMLAttributes<HTMLSpanElement>["class"];
  onClick?: JSX.CustomEventHandlersCamelCase<HTMLSpanElement>["onClick"];
}

export type {
  IActionsWrapper,
  IActionsWrapperAction,
  IBooleanFormField,
  IEngine,
  IEngineEdges,
  IEngineNodes,
  IEngineStore,
  IExplorerEdges,
  IExplorerElementRelativePosition,
  IExplorerElementRelativeSize,
  IExplorerNodes,
  IExplorerProject,
  IExplorerRootElement,
  IExplorerStore,
  IExplorerWorkflow,
  IExplorerWorkflowDisplayState,
  IExplorerWorkflowEdges,
  IExplorerWorkflowElement,
  IExplorerWorkflowElements,
  IExplorerWorkflowElementState,
  IExplorerWorkflowLayer,
  IExplorerWorkflowLayers,
  IExplorerWorkflowNode,
  IFormField,
  ILabel,
  INodeItem,
  INodeItemDisplayProps,
  INodeListItemProps,
  IProviderPropTypes,
  ITextFormField,
  IWorkspace,
};
