// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { BulletPoints } from "./BulletPoints";
import type { Domain } from "./Domain";
import type { Heading } from "./Heading";
import type { Link } from "./Link";
import type { OrderedPoints } from "./OrderedPoints";
import type { Paragraph } from "./Paragraph";
import type { SearchTerm } from "./SearchTerm";
import type { Table } from "./Table";
import type { TableRow } from "./TableRow";
import type { Title } from "./Title";
import type { WebPage } from "./WebPage";
import type { WorkflowStep } from "./WorkflowStep";

export type APIPayload =
  | { type: "Step"; data: WorkflowStep }
  | { type: "Domain"; data: Domain }
  | { type: "Link"; data: Link }
  | { type: "FileHTML"; data: WebPage }
  | { type: "Title"; data: Title }
  | { type: "Heading"; data: Heading }
  | { type: "Paragraph"; data: Paragraph }
  | { type: "BulletPoints"; data: BulletPoints }
  | { type: "OrderedPoints"; data: OrderedPoints }
  | { type: "Table"; data: Table }
  | { type: "TableRow"; data: TableRow }
  | { type: "Label"; data: string }
  | { type: "NamedEntity"; data: [string, string] }
  | { type: "SearchTerm"; data: SearchTerm };
