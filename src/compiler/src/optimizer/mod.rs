mod optimize_syntax_tree;
mod optimize_vm_instructions;

pub use self::{
    optimize_syntax_tree::optimize_syntax_tree_expression,
    optimize_vm_instructions::opimize_vm_instructions,
};
