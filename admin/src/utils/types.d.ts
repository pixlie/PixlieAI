import { JSX, JSXElement } from "solid-js";
import { Settings } from "../api_types/Settings";
import { SettingsStatus } from "../api_types/SettingsStatus";
import { Project } from "../api_types/Project";
import { APINodeItem } from "../api_types/APINodeItem";
import { Workspace } from "../api_types/Workspace";
import { APINodeEdges } from "../api_types/APINodeEdges.ts";
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

interface IPosition {
  x1: number;
  y1: number;
  x2: number;
  y2: number;
}

interface INodesAndEdges {
  nodes: APINodeItem[];
  edges: IEngineEdges;
  siblingNodes: Array<Array<number>>;

  canvasPosition: IPosition;
  nodePositions: Array<IPosition>;
}

interface IExplorerStore {
  projects: { [projectId: string]: INodesAndEdges };
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
  IProviderPropTypes,
  IWorkspace,
  IFormField,
  ITextFormField,
  IBooleanFormField,
  IEngine,
  IEngineEdges,
  IEngineNodes,
  IEngineStore,
  IPosition,
  IExplorerStore,
  INodeItem,
  INodeItemDisplayProps,
  INodeListItemProps,
  IActionsWrapper,
  IActionsWrapperAction,
  ILabel,
};
