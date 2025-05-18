// This is just a sketch for the proposal of handling our build-time rendering

/**
 * There are two invariants in our build-time rendering phase:
 * 1. ALL Server Values have to be replaced by special marks in the HTML output
 * 2. ALL HTML primitives existing in the user codebase MUST be pre-rendered to HTML
 *
 * This is very difficult to achieve in practise, our approach to this is a two-phase
 * process, first step consisting of hooking into the JSX transpiler to extract useful
 * metadata and move some code around, later to be used in the second phase - rendering.
 *
 * In this phase, the client code we're trying to render here has already been
 * transformed by our JSX transpilation interceptor. All PRIMITIVE createElements calls
 * has been replaced by __ruxyHtmlCreateElement(createElement(...), 'primitive-id')
 * with the same signature as `React.createElement` and one extra - primitive ID.
 *
 * There are some rules limiting the usage of this framework, which are inherent from the
 * constraints stemming from this pre-rendering mechanism.
 *
 * Usage rules:
 * 1. There are only two kinds of values that are allowed to be passed as props to HTML elements:
 *    - Server Values
 *    - Component-local values
 *
 *    Server Values are values that originate from the server, e.g. `const value = useServerValue(...)`.
 *
 *    Component-local Values are values that originate directly from user's code, e.g. `const someValue = 123`,
 *    **in the same component**. TODO: We can't guarantee this on the framework level, find another solution.
 *
 *    For example, this code is not allowed unless `someValue` is either a Server Value or a Static Value:
 *    ```
 *    <div prop1={someValue} />
 *    ```
 *
 *    However, for non-primitive React components, this rule does not apply, so you can safely write:
 *    ```
 *    <Whatever you={want.here} />
 *    ```
 *
 *    If you pass any other kind of value as a prop to any HTML primitive, the transpiler will throw
 *    a build-time error, outlining the problematic code.
 *
 *    There is one exception to this rule - *primary branch of execution*. If your HTML element is
 *    rendered unconditionally on the first run (meaning that when your entire app is being rendered,
 *    your element is rendered too), then this rule doesn't apply and you can pass whatever you want
 *    as a prop.
 *
 * 1. HTML Primitives cannot be rendered outside a special HTML wrapper `$(<fn>)`:
 *    e.g. `$(({ server }) => <div></div>)`. The returned element MUST be an HTML primitive.
 * */

type HtmlPrimitiveElement = {
  id: string;
  parentId: string;
  childrenIds: string;
  /**
   * @returns HTML
   * The transpiler will insert something like this here:
   * ```
   * const html = __ruxyHtmlCreateElement('div', { prop1: 'something.we.dont.know', ... }, 'childrenId1', ...)
   * ```
   *
   * The `'something.we.dont.know'` here is a stringified code that the user passed into the prop.
   * The __ruxyHtmlCreateElement function is responsible for "testing" the evaluation of that code
   * using try/catch to see if it can inlined as-is, or it needs to be rendered again in its parent
   * context. This goes recursively up the tree all the way to the React tree root.
   *
   * Of course the traversal should only happen once (from bottom to top) for all components.
   * This should work like this - render components that don't need parent context, store all
   * components that need parent context and move to the parent after all children has been evaluated.
   * */
  render: () => string;
};

declare const ___RUXY_HTML_PRIMITIVE_ELEMENTS: HtmlPrimitiveElement[];

/**
 * This is a replacement for React.createElement. Unlike the original function though,
 * this one is "moved" by the transpiler to the global array of these functions. In this
 * "renderer" script, we're NOT running the whole React tree. We only run these functions
 * in the array (meaning that we render ALL the primitive elements, otherwise some elements
 * wouldn't run because of conditional rendering).
 * */
// @ts-ignore
function __ruxyHtmlCreateElement(tagName: string, props: object, ...children: ReactNode[]) {
  // This is the place to pre-render the element and output it for the server to bake it into the binary
  // TODO: Run through all the props. Try to
}
