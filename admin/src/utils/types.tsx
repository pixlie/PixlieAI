import { JSX } from "solid-js";
import { Settings } from "../api_types/Settings";
import { SettingsStatus } from "../api_types/SettingsStatus";
import { Project } from "../api_types/Project";
import { APINodeItem } from "../api_types/APINodeItem";
import { Workspace } from "../api_types/Workspace";

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
  edges: { [nodeId: number]: Array<[number, string]> };
}

interface IEngineStore {
  projects: { [projectId: string]: IEngine };
}

type IFormFieldValue = string | number | Array<string> | undefined;

type DisplayAs = "Drawer" | "MainContent";

interface INodeItemDisplayProps {
  nodeId: number;
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
  isRequired?: boolean;
  isEditable?: boolean;
  autocomplete?: boolean;
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
};
