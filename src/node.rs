use std::cell::RefCell;

use emacs::{defun, Value, Result};

use tree_sitter::InputEdit;

use crate::types::{WrappedNode, Range, Point};

/// Exposes methods that return a node's property.
macro_rules! defun_node_props {
    ($($(#[$meta:meta])* $($lisp_name:literal)? fn $name:ident -> $type:ty $(; $into:ident)? )*) => {
        $(
            #[defun$((name = $lisp_name))?]
            $(#[$meta])*
            fn $name(node: &WrappedNode) -> Result<$type> {
                Ok(node.inner().$name()$(.$into())?)
            }
        )*
    };
}

/// Exposes methods that return another node.
macro_rules! defun_node_navs {
    ($($(#[$meta:meta])* $($lisp_name:literal)? fn $name:ident $( ( $( $param:ident $($into:ident)? : $type:ty ),* ) )?)*) => {
        $(
            #[defun$((name = $lisp_name))?]
            $(#[$meta])*
            fn $name(node: &WrappedNode, $( $( $param : $type ),* )? ) -> Result<Option<RefCell<WrappedNode>>> {
                Ok(node.inner().$name( $( $( $param $(.$into())? ),* )? ).map(|other| {
                    RefCell::new(unsafe { node.wrap(other) })
                }))
            }
        )*
    };
}

defun_node_props! {
/// Return NODE's type-id.
"node-type-id" fn kind_id -> u16
/// Return NODE's type.
"node-type" fn kind -> &'static str

/// Return t if NODE is named.
/// Named nodes correspond to named rules in the grammar, whereas anonymous nodes
/// correspond to string literals in the grammar.
"node-named-p" fn is_named -> bool
"node-extra-p" fn is_extra -> bool
"node-error-p" fn is_error -> bool
/// Return t if NODE is missing.
/// Missing nodes are inserted by the parser in order to recover from certain kinds
/// of syntax errors.
"node-missing-p" fn is_missing -> bool
/// Return t if NODE has been edited.
"node-has-changes-p" fn has_changes -> bool
/// Return t if NODE is a syntax error or contains any syntax errors.
"node-has-error-p" fn has_error -> bool

/// Return NODE's start byte.
"node-start-byte" fn start_byte -> usize
/// Return NODE's start point.
"node-start-point" fn start_position -> Point; into
/// Return NODE's end byte.
"node-end-byte" fn end_byte -> usize
/// Return NODE's end point.
"node-end-point" fn end_position -> Point; into
/// Return NODE's [start-point end-point].
"node-range" fn range -> Range; into

/// Return NODE's number of children.
"count-children" fn child_count -> usize
/// Return NODE's number of named children.
"count-named-children" fn named_child_count -> usize
}

/// Apply FUNCTION to each of NODE's children, for side effects only.
#[defun]
fn mapc_children(function: Value, node: &WrappedNode) -> Result<()> {
    for child in node.inner().children() {
        let child = RefCell::new(unsafe { node.wrap(child) });
        function.call((child,))?;
    }
    Ok(())
}

defun_node_navs! {
/// Return NODE's child at the given zero-based index.
"get-nth-child" fn child(i: usize)
/// Return NODE's named child at the given zero-based index.
"get-nth-named-child" fn named_child(i: usize)
/// Return NODE's child with the given FIELD-NAME.
"get-child-by-field-name" fn child_by_field_name(field_name: String)
/// Return NODE's child with the given numerical FIELD-ID.
"get-child-by-field-id" fn child_by_field_id(field_id: u16)

/// Return NODE's parent node.
"get-parent" fn parent

/// Return NODE's next sibling.
"get-next-sibling" fn next_sibling
/// Return NODE's previous sibling.
"get-prev-sibling" fn prev_sibling
/// Return NODE's next named sibling.
"get-next-named-sibling" fn next_named_sibling
/// Return NODE's previous named sibling.
"get-prev-named-sibling" fn prev_named_sibling

/// Return the smallest node within NODE that spans the given range of bytes.
"get-descendant-for-byte-range" fn descendant_for_byte_range(start: usize, end: usize)
/// Return the smallest node within NODE that spans the given range of points.
"get-descendant-for-point-range" fn descendant_for_point_range(start into: Point, end into: Point)
/// Return the smallest named node within NODE that spans the given range of bytes.
"get-named-descendant-for-byte-range" fn named_descendant_for_byte_range(start: usize, end: usize)
/// Return the smallest named node within NODE that spans the given range of points.
"get-named-descendant-for-point-range" fn named_descendant_for_point_range(start into: Point, end into: Point)
}

defun_node_props! {
/// Return the sexp representation of NODE, in a string.
"node-to-sexp" fn to_sexp -> String
}

/// Edit NODE to keep it in sync with source code that has been edited.
///
/// This function is only rarely needed. When you edit a syntax tree, all of the
/// nodes that you retrieve from the tree afterward will already reflect the edit.
/// You only need to use this function when you have a node that you want to keep
/// and continue to use after an edit.
///
/// Note that indexing must be zero-based.
#[defun]
fn edit_node(
    node: &mut WrappedNode,
    start_byte: usize,
    old_end_byte: usize,
    new_end_byte: usize,
    start_point: Point,
    old_end_point: Point,
    new_end_point: Point,
) -> Result<()> {
    let edit = InputEdit {
        start_byte,
        old_end_byte,
        new_end_byte,
        start_position: start_point.into(),
        old_end_position: old_end_point.into(),
        new_end_position: new_end_point.into(),
    };
    node.inner_mut().edit(&edit);
    Ok(())
}
