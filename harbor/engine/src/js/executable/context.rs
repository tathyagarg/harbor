use crate::js::executable::realm::Realm;

pub enum ScriptOrModule {
    Script,
    Module,
}

pub struct ExecutionContext {
    pub realm: Realm,
    pub script_or_module: ScriptOrModule,
}
