use wllvm::attribute::AttrKind;

use crate::{
    codegen::{error, types::Type, CodegenUnit},
    error_handling::Diagnostic,
};

impl<'ctx> CodegenUnit<'_, 'ctx> {
    pub fn generate_entrypoint(&self) -> Result<(), Diagnostic> {
        let main = self
            .c
            .name_store
            .get_item_from_string(&format!("{}::main", self.crate_name))
            .unwrap()
            .as_function()
            .unwrap();

        let llvm_function = self.module.add_function(
            c"_start",
            self.c
                .context
                .fn_type(*self.c.context.void_type(), &[], false),
        );

        llvm_function.add_attribute(self.c.context.attribute(AttrKind::NoReturn()));
        llvm_function.add_attribute(self.c.context.attribute(AttrKind::NoUnwind()));

        let old_block = self.builder.current_block();

        let fn_block = llvm_function.add_basic_block(c"");
        self.builder.position_at_end(fn_block);

        self.builder.build_fn_call(main.function, &[], c"");

        let Some(exit_fn) = self
            .c
            .name_store
            .get_item_from_string("std::exit")
            .and_then(|i| i.as_function())
        else {
            return Err(error::no_exit(self.crate_name));
        };

        if &exit_fn.signature.params != &[Type::i(32)] {
            return Err(error::exit_arguments());
        }

        let exit_fn_name = exit_fn.function.name();

        let exit_fn = self.module.get_function(exit_fn_name).unwrap_or_else(|| {
            let exit_fn = self.module.add_function(c"", exit_fn.function.type_());
            exit_fn.set_name(exit_fn_name);
            exit_fn
        });

        self.builder.build_fn_call(
            exit_fn,
            &[*self.c.context.int_type(32).const_(0, false)],
            c"",
        );

        self.builder.build_unreachable();

        if let Some(old_block) = old_block {
            self.builder.position_at_end(old_block)
        }

        Ok(())
    }
}
