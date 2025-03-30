import { JSX } from "solid-js";
import { Settings } from "../api_types/Settings";
import { SettingsStatus } from "../api_types/SettingsStatus";
import { Project } from "../api_types/Project";
import { APINodeItem } from "../api_types/APINodeItem";
import { Workspace } from "../api_types/Workspace";
import { APINodeEdges } from "../api_types/APINodeEdges.ts";

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

interface INodeItemDisplayProps {
  nodeId: number;
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

export type {
  IProviderPropTypes,
  IWorkspace,
  IFormField,
  ITextFormField,
  IBooleanFormField,
  IEngine,
  IEngineStore,
  INodeItem,
  INodeItemDisplayProps,
};
