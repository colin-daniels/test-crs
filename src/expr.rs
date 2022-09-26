pub enum Scheme {}

impl Scheme {
    pub fn get_field_count(&self) -> usize {
        todo!()
    }
}

pub struct ExecutionContext<'e> {
    scheme: &'e Scheme,
    values: Box<[Option<String>]>,
}

impl<'e> ExecutionContext<'e> {
    /// Creates an execution context associated with a given scheme.
    ///
    /// This scheme will be used for resolving any field names and indices.
    pub fn new<'s: 'e>(scheme: &'s Scheme) -> Self {
        ExecutionContext {
            scheme,
            values: vec![None; scheme.get_field_count()].into(),
        }
    }
}

trait Expr<'s> {
    fn compile(self) -> CompiledExpr<'s>;
}

pub(crate) struct CompiledExpr<'s>(Box<dyn 's + Fn(&ExecutionContext<'s>) -> bool>);

impl<'s> CompiledExpr<'s> {
    /// Creates a compiled expression IR from a generic closure.
    pub(crate) fn new(closure: impl 's + Fn(&ExecutionContext<'s>) -> bool) -> Self {
        CompiledExpr(Box::new(closure))
    }

    /// Executes a filter against a provided context with values.
    pub fn execute(&self, ctx: &ExecutionContext<'s>) -> bool {
        self.0(ctx)
    }
}
