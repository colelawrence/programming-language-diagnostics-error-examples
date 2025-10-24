import { gen } from "./gen";

/**
 * For example `#[codegen(fn = "timeline() -> Timeline")]` over any declarations.
 */
export function parseRouterFns(input: gen.Input, errors: gen.OutputMessage[]) {
  const fns: {
    key: string;
    decl: gen.InputDeclaration;
    inputFmt: gen.Format;
    responseFmt: gen.Format;
  }[] = [];

  // console.error("Number of declarations: ", input.declarations.length);
  for (const decl of input.declarations) {
    // For example `#[codegen(fn = "timeline(): Timeline")]`
    if (decl.codegen_attrs?.fn) {
      const { fn } = decl.codegen_attrs;
      const RE = /^([\w$]+)\(\) -> ([\w$]+|\(\))$/;
      const match = RE.exec(decl.codegen_attrs.fn[0]);
      if (match) {
        const [key, responseType] = match.slice(1);
        const inputFmt = gen.Format.TypeName({
          ident: ident(decl.id),
          generics: [], // TODO?
        });
        const responseFmt = responseType === "()" ? gen.Format.Unit() : gen.Format.TypeName({ ident: ident(responseType), generics: [] });
        fns.push({ key, decl, inputFmt, responseFmt });
      } else {
        errors.push(
          gen.OutputMessage({
            message: `Invalid codegen fn format: ${decl.codegen_attrs.fn[0]}`,
            labels: [fn],
          }),
        );
      }
    }
  }

  return fns;
}
function ident(id: string): string {
  return id.replace(/[^a-zA-Z0-9$_]/g, "$").replace(/^(\d)/, "$$1");
}
