#![deny(warnings)]
#![feature(plugin_registrar)]
#![feature(rustc_private)]

extern crate rustc_plugin;
extern crate syntax;

use rustc_plugin::registry::Registry;
use syntax::ast::{Item, ItemKind, MetaItem, Ident, self};
use syntax::codemap::{Span, self};
use syntax::ext::base::{ExtCtxt, MultiModifier, Annotatable};
use syntax::ext::build::AstBuilder;
use syntax::ptr::P;
use syntax::symbol::Symbol;

#[doc(hidden)]
#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_syntax_extension(
        Symbol::intern("criterion"),
        MultiModifier(Box::new(expand_meta_criterion)));
}

/// Expands the `#[criterion]` attribute
///
/// Expands:
///
/// ```
/// #[criterion]
/// fn routine(b: &mut Bencher) {
///     b.iter(|| {});
/// }
/// ```
///
/// Into:
///
/// ```
/// #[test]
/// fn routine() {
///     fn routine(b: &mut Bencher) {
///         b.iter(|| {});
///     }
///
///     ::criterion::Criterion::default().bench_function("routine", routine);
/// }
/// ```
fn expand_meta_criterion(
    cx: &mut ExtCtxt,
    span: Span,
    _: &MetaItem,
    item: Annotatable,
) -> Annotatable {
    if let Annotatable::Item(item) = item {
        if let ItemKind::Fn(..) = item.node {
            // Copy original function without attributes
            let routine = P(Item { attrs: Vec::new(), .. (*item).clone() });
            
            // `::criterion::Criterion::default()`
            let crate_ident = Ident::from_str("criterion");
            let struct_ident = Ident::from_str("Criterion");
            let method_ident = Ident::from_str("default");
            let criterion_path = vec!(crate_ident, struct_ident, method_ident);
            let default_criterion = cx.expr_call_global(span, criterion_path, vec!());

            // `.bench_function("routine", routine);`
            let routine_str = cx.expr_str(span, routine.ident.name);
            let routine_ident = cx.expr_ident(span, routine.ident);
            let bench_ident = Ident::from_str("bench_function");
            let bench_call = cx.expr_method_call(span, default_criterion, bench_ident,
                                                 vec!(routine_str, routine_ident));
            let bench_call = cx.stmt_expr(bench_call).add_trailing_semicolon();

            // Wrap original function + bench call in a test function
            let fn_decl = cx.stmt_item(span, routine);
            let body = cx.block(span, vec!(fn_decl, bench_call));
            let nil = P(ast::Ty {
                id: ast::DUMMY_NODE_ID,
                node: ast::TyKind::Tup(vec![]),
                span: codemap::DUMMY_SP,
            });
            let test = cx.item_fn(span, item.ident, Vec::new(), nil, body);

            // Add the `#[test]` attribute to existing attributes
            let mut attrs = item.attrs.clone();
            attrs.
                push(cx.attribute(span, cx.meta_word(span, Symbol::gensym("test"))));

            Annotatable::Item(P(Item { attrs: attrs, .. (*test).clone() }))
        } else {
            cx.span_err(span, "#[criterion] only supported on functions");
            Annotatable::Item(item)
        }
    } else {
        cx.span_err(span, "#[criterion] only supported on functions");
        item
    }

}
