import { Select as SelectPrimitive } from "bits-ui";

export { default as SelectTrigger } from "./select-trigger.svelte";
export { default as SelectContent } from "./select-content.svelte";
export { default as SelectItem } from "./select-item.svelte";

export const Select = SelectPrimitive.Root;
export const SelectValue = SelectPrimitive.Value;
