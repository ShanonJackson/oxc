use std::{borrow::Cow, fmt::Write, sync::OnceLock};

pub fn offline_fixture(filename: &str) -> Option<Cow<'static, str>> {
    match filename {
        "RadixUIAdoptionSection.jsx" => Some(Cow::Borrowed(radix_ui_adoption_section())),
        "react.development.js" => Some(Cow::Borrowed(react_development_js())),
        "cal.com.tsx" => Some(Cow::Borrowed(cal_com_tsx())),
        "binder.ts" => Some(Cow::Borrowed(binder_ts())),
        _ => None,
    }
}

fn stash(source: String, slot: &OnceLock<&'static str>) -> &'static str {
    slot.get_or_init(|| Box::leak(source.into_boxed_str()))
}

fn radix_ui_adoption_section() -> &'static str {
    static CACHE: OnceLock<&'static str> = OnceLock::new();

    let mut out = String::from("import * as React from 'react';\n");
    out.push_str("const highlights = [\n");
    out.push_str("  { title: 'Accessible', description: 'Radix primitives embrace accessibility by default.' },\n");
    out.push_str("  { title: 'Composable', description: 'APIs stay close to platform primitives for ergonomic composition.' },\n");
    out.push_str("  { title: 'Themeable', description: 'Design tokens and CSS variables make customization straightforward.' },\n");
    out.push_str("];\n\n");
    out.push_str("export function RadixUIAdoptionSection() {\n");
    out.push_str("  const [expanded, setExpanded] = React.useState(null);\n");
    out.push_str("  const toggle = React.useCallback(title => {\n");
    out.push_str("    setExpanded(prev => (prev === title ? null : title));\n");
    out.push_str("  }, []);\n");
    out.push_str("  return (\n");
    out.push_str("    <section data-component=\"radix-ui-adoption\">\n");
    out.push_str("      <header>\n");
    out.push_str("        <h1>Build with confidence</h1>\n");
    out.push_str("        <p>Modern teams ship fast with Radix primitives.</p>\n");
    out.push_str("      </header>\n");
    out.push_str("      <ul>\n");
    out.push_str("        {highlights.map(highlight => (\n");
    out.push_str("          <li key={highlight.title}>\n");
    out.push_str("            <button\n");
    out.push_str("              type=\"button\"\n");
    out.push_str("              aria-expanded={expanded === highlight.title}\n");
    out.push_str("              onClick={() => toggle(highlight.title)}\n");
    out.push_str("            >\n");
    out.push_str("              <span>{highlight.title}</span>\n");
    out.push_str("            </button>\n");
    out.push_str("            {expanded === highlight.title ? (\n");
    out.push_str("              <article>\n");
    out.push_str("                <p>{highlight.description}</p>\n");
    out.push_str("                <footer>\n");
    out.push_str("                  <a href=\"https://www.radix-ui.com\">Explore Radix UI</a>\n");
    out.push_str("                </footer>\n");
    out.push_str("              </article>\n");
    out.push_str("            ) : null}\n");
    out.push_str("          </li>\n");
    out.push_str("        ))}\n");
    out.push_str("      </ul>\n");
    out.push_str("    </section>\n");
    out.push_str("  );\n");
    out.push_str("}\n\n");
    out.push_str("export function useRadixHighlights() {\n");
    out.push_str("  return React.useMemo(() => highlights.map(item => ({ ...item })), []);\n");
    out.push_str("}\n");
    stash(out, &CACHE)
}

fn react_development_js() -> &'static str {
    static CACHE: OnceLock<&'static str> = OnceLock::new();

    let mut out = String::with_capacity(1_600_000);
    out.push_str("// Offline React development fixture for benchmarks.\n");
    out.push_str("const REACT_ELEMENT_TYPE = Symbol.for('react.element');\n");
    out.push_str("export const Fragment = Symbol.for('react.fragment');\n");
    out.push_str("export function jsx(type, config, maybeKey, ...children) {\n");
    out.push_str("  const props = { ...config };\n");
    out.push_str("  props.children = children.length === 1 ? children[0] : children;\n");
    out.push_str("  return { $$typeof: REACT_ELEMENT_TYPE, type, key: maybeKey ?? null, ref: null, props };\n");
    out.push_str("}\n");
    out.push_str("export { jsx as jsxs, jsx as jsxDEV };\n");
    out.push_str("export function useState(initial) {\n");
    out.push_str("  let value = typeof initial === 'function' ? initial() : initial;\n");
    out.push_str("  const setValue = next => {\n");
    out.push_str("    value = typeof next === 'function' ? next(value) : next;\n");
    out.push_str("    return value;\n");
    out.push_str("  };\n");
    out.push_str("  return [() => value, setValue];\n");
    out.push_str("}\n");
    out.push_str("export function useMemo(factory, deps) {\n");
    out.push_str("  const cache = { deps, value: factory() };\n");
    out.push_str("  return cache.value;\n");
    out.push_str("}\n");
    out.push_str("function identity(value) {\n");
    out.push_str("  return value;\n");
    out.push_str("}\n");
    for i in 0..2560 {
        let _ = writeln!(out, "export function Component{0}(props) {{", i);
        out.push_str("  const [getValue, setValue] = useState(() => props.seed ?? 0);\n");
        out.push_str("  const derived = useMemo(() => ({\n");
        out.push_str("    id: props.id ?? getValue(),\n");
        out.push_str("    label: props.label ?? 'component',\n");
        out.push_str("  }), [props.id, props.label]);\n");
        out.push_str("  if (props.onRender) {\n");
        out.push_str("    props.onRender(derived);\n");
        out.push_str("  }\n");
        out.push_str("  const entries = [];\n");
        out.push_str("  for (let j = 0; j < 4; j += 1) {\n");
        out.push_str("    entries.push(identity({ index: j, derived }));\n");
        out.push_str("  }\n");
        out.push_str("  return jsx('section', {\n");
        out.push_str("    role: props.role ?? 'region',\n");
        out.push_str("    ['data-component']: `component-");
        let _ = write!(out, "{i}`\n");
        out.push_str("  }, undefined, entries);\n");
        out.push_str("}\n");
        let _ = writeln!(out, "Component{0}.defaultProps = {{ role: 'region' }};", i);
    }
    stash(out, &CACHE)
}

fn cal_com_tsx() -> &'static str {
    static CACHE: OnceLock<&'static str> = OnceLock::new();

    let mut out = String::with_capacity(1_000_000);
    out.push_str("import { useEffect, useMemo, useState } from 'react';\n");
    out.push_str("type Slot<T = Date> = {\n");
    out.push_str("  id: string;\n  start: T;\n  end: T;\n  resources: readonly string[];\n};\n");
    out.push_str("type CalendarProps<T extends Slot> = {\n");
    out.push_str("  slots: readonly T[];\n  timezone?: string;\n  onSelect?(slot: T): void;\n  renderSlot?(slot: T): JSX.Element;\n};\n");
    out.push_str("export function useSlots<T extends Slot>(slots: readonly T[]) {\n");
    out.push_str("  const [filter, setFilter] = useState<string | null>(null);\n");
    out.push_str("  const filtered = useMemo(() => {\n");
    out.push_str("    if (!filter) return [...slots];\n");
    out.push_str("    return slots.filter(slot => slot.resources.includes(filter));\n");
    out.push_str("  }, [slots, filter]);\n");
    out.push_str("  return { filtered, filter, setFilter };\n");
    out.push_str("}\n");
    out.push_str("function formatRange(start: Date, end: Date, tz?: string) {\n");
    out.push_str("  const fmt = new Intl.DateTimeFormat('en', {\n");
    out.push_str("    hour: 'numeric', minute: '2-digit', timeZone: tz ?? 'UTC'\n  });\n");
    out.push_str("  return `${fmt.format(start)} – ${fmt.format(end)}`;\n}\n");
    for i in 0..640 {
        let _ = writeln!(
            out,
            "export function CalendarView{0}<T extends Slot>(props: CalendarProps<T>) {{",
            i
        );
        out.push_str("  const { slots, timezone, onSelect, renderSlot } = props;\n");
        out.push_str("  const { filtered, filter, setFilter } = useSlots(slots);\n");
        out.push_str("  const [selection, setSelection] = useState<T | null>(null);\n");
        out.push_str("  useEffect(() => {\n");
        out.push_str("    if (selection && onSelect) onSelect(selection);\n");
        out.push_str("  }, [selection, onSelect]);\n");
        out.push_str("  const handleSelect = (slot: T) => {\n");
        out.push_str("    setSelection(slot);\n");
        out.push_str("  };\n");
        out.push_str("  const summary = useMemo(() => ({\n");
        out.push_str("    count: filtered.length,\n    resources: Array.from(new Set(filtered.flatMap(s => s.resources)))\n  }), [filtered]);\n");
        out.push_str("  return (\n");
        out.push_str("    <section data-view=\"cal-com\">\n");
        out.push_str("      <header>\n");
        out.push_str("        <h2>Available slots</h2>\n");
        out.push_str("        <p>{summary.count} matches</p>\n");
        out.push_str("      </header>\n");
        out.push_str("      <div className=\"filters\">\n");
        out.push_str("        <label>Resource filter</label>\n");
        out.push_str("        <select\n");
        out.push_str("          value={filter ?? ''}\n");
        out.push_str("          onChange={event => setFilter(event.target.value || null)}\n");
        out.push_str("        >\n");
        out.push_str("          <option value=\"\">All resources</option>\n");
        out.push_str("          {summary.resources.map(resource => (\n");
        out.push_str("            <option key={resource}>{resource}</option>\n");
        out.push_str("          ))}\n");
        out.push_str("        </select>\n");
        out.push_str("      </div>\n");
        out.push_str("      <ul className=\"slots\">\n");
        out.push_str("        {filtered.map(slot => (\n");
        out.push_str("          <li key={slot.id}>\n");
        out.push_str("            <button type=\"button\" onClick={() => handleSelect(slot)}>\n");
        out.push_str("              <span>{formatRange(slot.start, slot.end, timezone)}</span>\n");
        out.push_str("              <small>{slot.resources.join(', ')}</small>\n");
        out.push_str("            </button>\n");
        out.push_str("            {renderSlot ? renderSlot(slot) : null}\n");
        out.push_str("          </li>\n");
        out.push_str("        ))}\n");
        out.push_str("      </ul>\n");
        out.push_str("    </section>\n");
        out.push_str("  );\n");
        out.push_str("}\n");
    }
    stash(out, &CACHE)
}

fn binder_ts() -> &'static str {
    static CACHE: OnceLock<&'static str> = OnceLock::new();

    let mut out = String::with_capacity(600_000);
    out.push_str("export type BinderSymbol = string & { readonly __binder: unique symbol };\n");
    out.push_str("export interface BindingFlags {\n");
    out.push_str("  exported: boolean;\n  ambient: boolean;\n  optional: boolean;\n}\n");
    out.push_str("export interface BinderNode {\n");
    out.push_str("  id: BinderSymbol;\n  parent?: BinderNode;\n  flags: BindingFlags;\n  children: BinderNode[];\n}\n");
    out.push_str("export function createBinder(id: string, parent?: BinderNode): BinderNode {\n");
    out.push_str("  return {\n");
    out.push_str("    id: id as BinderSymbol,\n");
    out.push_str("    parent,\n");
    out.push_str("    flags: { exported: false, ambient: false, optional: false },\n");
    out.push_str("    children: [],\n");
    out.push_str("  };\n");
    out.push_str("}\n");
    out.push_str("export function bindChild(parent: BinderNode, child: BinderNode) {\n");
    out.push_str("  parent.children.push(child);\n");
    out.push_str("  child.parent = parent;\n");
    out.push_str("}\n");
    for i in 0..2048 {
        let _ = writeln!(out, "export interface Declaration{0} {{", i);
        out.push_str("  name: string;\n  members: readonly string[];\n  metadata?: Record<string, unknown>;\n}\n");
        let _ = writeln!(
            out,
            "export function bindDeclaration{0}(root: BinderNode, decl: Declaration{0}) {{",
            i
        );
        out.push_str("  const scope = createBinder(decl.name, root);\n");
        out.push_str("  scope.flags = {\n");
        out.push_str("    exported: decl.members.includes('export'),\n");
        out.push_str("    ambient: decl.members.includes('declare'),\n");
        out.push_str("    optional: decl.members.includes('optional'),\n");
        out.push_str("  };\n");
        out.push_str("  bindChild(root, scope);\n");
        out.push_str("  for (const member of decl.members) {\n");
        out.push_str("    const node = createBinder(`${decl.name}.${member}`, scope);\n");
        out.push_str("    bindChild(scope, node);\n");
        out.push_str("  }\n");
        out.push_str("}\n");
    }
    stash(out, &CACHE)
}
