import { JSX } from "solid-js";
import { Settings } from "../api_types/Settings";
import { SettingsStatus } from "../api_types/SettingsStatus";
import { Node } from "../api_types/Node";

interface IProviderPropTypes {
  children: JSX.Element;
}

interface IWorkspace {
  settings?: Settings;
  settingsStatus?: SettingsStatus;

  isReady: boolean;
  isFetching: boolean;
}

interface IEngine {
  nodes: { [nodeId: number]: Node };
  nodeIdsByLabel: { [label: string]: Array<number> };

  isReady: boolean;
  isFetching: boolean;
}

type IFormFieldValue = string | number | Array<string> | undefined;

interface IFormField {
  name: string;
  label?: string | null;
  description?: string | null;
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
};
