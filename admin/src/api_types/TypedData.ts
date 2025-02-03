// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { LossyLocation } from "./LossyLocation";

export type TypedData =
  | { SmallInteger: number }
  | { Integer: number }
  | { Float: number }
  | { String: string }
  | { Boolean: boolean }
  | { Date: string }
  | { Time: string }
  | { DateTime: string }
  | { Email: string }
  | { Link: string }
  | { Currency: string }
  | { Place: LossyLocation };
