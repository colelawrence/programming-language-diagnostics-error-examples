import { readFileSync } from "node:fs";
import { gen, Code } from "./gen";
import { parseRouterFns } from "./parseRouterFns";

// check if this file is being run directly based on args
if (process.argv[1]?.includes("generateRustRouterSimple.ts")) {
  setTimeout(() => {
    const arg = process.argv.slice(2)[0];
    const payload = arg ?? readFileSync(0, "utf8");
    const input = JSON.parse(payload) as gen.Input;
    const result = generateRust(input, {
      fileName: "router_gen.rs",
    });
    console.log(JSON.stringify(result));
  });
}

export function generateRust(
  input: gen.Input,
  args: {
    /** What to call the file with all the declarations */
    fileName: string;
  },
): gen.Output {
  Code.docStringSettings.multi_line = {
    prefix: "",
    empty_line_pre: "///",
    line_pre: "\n/// ",
    suffix: "",
  };
  Code.docStringSettings.single_line = { prefix: "/// ", suffix: "" };
  Code.docStringSettings.skip_rust_attrs = true;

  const errors: gen.OutputMessage[] = [];
  const fns = parseRouterFns(input, errors);
  const seenFmts = new Set<string>();
  
  const output = Code.group([
    `use super::{ObserverImpl, WireResponseSender};`,
    `use crate::context::Context;`,
    `use crate::*;`,
    `use serde::{Deserialize, Serialize};`,
    ``,
    `pub trait CallHandler {`,
    Code.indented([
      ...fns.flatMap((fn) => [
        `fn ${fn.key}(`,
        Code.indented([
          `&self,`,
          `ctx: &Context,`,
          `params: ${createFormat(fn.inputFmt).src},`,
          `tx: ObserverImpl<${createFormat(fn.responseFmt).src}>,`
        ]),
        `);`,
      ]),
    ]),
    `}`,
    ``,
    `#[allow(non_camel_case_types)]`,
    `#[derive(Serialize, Deserialize, Debug, Clone)]`,
    `pub enum CallGen {`,
    Code.indented(fns.map((fn) => `${fn.key}(${createFormat(fn.inputFmt).src}),`)),
    `}`,
    ``,
    `#[allow(non_camel_case_types)]`,
    `#[derive(Serialize, Deserialize, Debug, Clone)]`,
    `pub enum ResponseNextGen {`,
    Code.indented(fns.map((fn) => `${fn.key}(${createFormat(fn.responseFmt).src}),`)),
    `}`,
    ``,
    `pub(crate) fn gen_call(`,
    Code.indented([
      `ctx: &Context,`,
      `id: usize,`,
      `call: CallGen,`,
      `handler: &dyn CallHandler,`,
      `sender: Box<dyn WireResponseSender>,`,
    ]),
    `) {`,
    Code.indented([
      `match call {`,
      Code.indented(
        fns.flatMap((fn) => [
          `CallGen::${fn.key}(params) => handler.${fn.key}(`,
          Code.indented([
            `ctx,`,
            `params,`,
            `ObserverImpl::new(id, sender),`,
          ]),
          `),`,
        ]),
      ),
      `}`,
    ]),
    `}`,
    ``,
    `// ToResponseNextGen implementations`,
    ...fns
      .filter((fn) => {
        const fmt = createFormat(fn.responseFmt).src;
        if (seenFmts.has(fmt)) {
          return false;
        }
        seenFmts.add(fmt);
        return true;
      })
      .flatMap((fn) => [
        `impl super::ToResponseNextGen for ${createFormat(fn.responseFmt).src} {`,
        Code.indented([
          `fn to_response_next_gen(self) -> ResponseNextGen {`,
          Code.indented([
            `ResponseNextGen::${fn.key}(self)`,
          ]),
          `}`,
        ]),
        `}`,
        ``,
      ]),
  ]);

  return {
    errors,
    files: [{
      path: args.fileName,
      source: output.toStringIndented("    ", 0),
    }],
    warnings: [],
  };
}

function createFormat(
  fmt: gen.Format,
): { src: string } {
  return gen.Format.apply<{ src: string }>({
    Unit: () => ({ src: "()" }),
    TypeName: (value) => ({
      src: value.ident,
    }),
    // Add other cases as needed
    I8: () => ({ src: "i8" }),
    I16: () => ({ src: "i16" }),
    I32: () => ({ src: "i32" }),
    I64: () => ({ src: "i64" }),
    U8: () => ({ src: "u8" }),
    U16: () => ({ src: "u16" }),
    U32: () => ({ src: "u32" }),
    U64: () => ({ src: "u64" }),
    F32: () => ({ src: "f32" }),
    F64: () => ({ src: "f64" }),
    Bool: () => ({ src: "bool" }),
    String: () => ({ src: "String" }),
    Vec: (inner) => ({ src: `Vec<${createFormat(inner).src}>` }),
    Option: (inner) => ({ src: `Option<${createFormat(inner).src}>` }),
  })(fmt);
}
