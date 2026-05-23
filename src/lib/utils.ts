import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

// merges conditional class names and resolves tailwind conflicts in one pass.
export function cn(...inputs: ClassValue[]): string {
  return twMerge(clsx(inputs));
}
