/**
 * The `html` tagged template: htm bound to Preact's h().
 *
 * htm's bundled types don't survive Deno's CJS interop — describe the
 * surface we use once, here, instead of in every component.
 */
import { h, type VNode } from "preact";
import htm from "htm";

const bind = htm as unknown as {
  bind(
    hFn: typeof h,
  ): (strings: TemplateStringsArray, ...values: unknown[]) => VNode;
};

export const html = bind.bind(h);
