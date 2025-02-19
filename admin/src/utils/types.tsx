import { JSX } from "solid-js";
import { Settings } from "../api_types/Settings";
import { SettingsStatus } from "../api_types/SettingsStatus";
import { Project } from "../api_types/Project";
import { APINodeItem } from "../api_types/APINodeItem.ts";

interface IProviderPropTypes {
  children: JSX.Element;
}

interface IWorkspace {
  settings?: Settings;
  settingsStatus?: SettingsStatus;
  projects?: Array<Project>;

  isReady: boolean;
  isFetching: boolean;
}

interface INodeItem extends APINodeItem {
  isFetching: boolean;
}

interface IEngine {
  projectId?: string;
  nodes: { [nodeId: number]: INodeItem };
  nodeIdsByLabel: { [label: string]: Array<number> };
}

type IFormFieldValue = string | number | Array<string> | undefined;

type DisplayAs = "Drawer" | "MainContent";

interface IFormField {
  name: string;
  placeholder?: string | null;
  size?: "xs" | "sm" | "base" | "lg";
  displayBlock?: boolean;
  value?: IFormFieldValue;
  onChange?: (name: string, value: IFormFieldValue) => void;
  onFocus?: () => void;
  isRequired?: boolean;
  isEditable?: boolean;
}

export type {
  IProviderPropTypes,
  IWorkspace,
  IFormField,
  IFormFieldValue,
  IEngine,
  INodeItem,
  DisplayAs,
};
