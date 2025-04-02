// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { Link } from "./Link";
import type { ProjectSettings } from "./ProjectSettings";
import type { TableRow } from "./TableRow";

export type APIPayload =
  | { type: "Link"; data: Link }
  | { type: "Text"; data: string }
  | { type: "Tree"; data: string }
  | { type: "TableRow"; data: TableRow }
  | { type: "ProjectSettings"; data: ProjectSettings };
