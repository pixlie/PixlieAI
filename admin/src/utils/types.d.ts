import { JSX, JSXElement } from "solid-js";
import { Settings } from "../api_types/Settings";
import { SettingsStatus } from "../api_types/SettingsStatus";
import { Project } from "../api_types/Project";
import { APINodeItem } from "../api_types/APINodeItem";
import { Workspace } from "../api_types/Workspace";
import { APINodeEdges } from "../api_types/APINodeEdges.ts";
import { TailwindClassKeyType } from "./uIClasses/types";

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

interface IEngine {
  nodes: { [nodeId: number]: INodeItem };
  edges: { [nodeId: number]: APINodeEdges };
  nodesFetchedAt: number;
  edgesFetchedAt: number;
  isFetching: boolean;
}

interface IEngineStore {
  projects: { [projectId: string]: IEngine };
  sync: Array<string>;
}

type IFormFieldValue = string | number | Array<string> | undefined;

type DisplayAs = "Drawer" | "MainContent";

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
  value?: IFormFieldValue;
  onChange?: (name: string, value: IFormFieldValue) => void;
  onFocus?: () => void;
  onKeyUp?: (event: KeyboardEvent) => void;
  isRequired?: boolean;
  isEditable?: boolean;
  autocomplete?: boolean;
}

interface IActionsWrapperAction {
  render: boolean;
  color?: TailwindClassKeyType;
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
  color: TailwindClassKeyType;
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
  IFormFieldValue,
  IEngine,
  IEngineStore,
  INodeItem,
  DisplayAs,
  INodeItemDisplayProps,
  INodeListItemProps,
  IActionsWrapper,
  IActionsWrapperAction,
  ILabel,
};
