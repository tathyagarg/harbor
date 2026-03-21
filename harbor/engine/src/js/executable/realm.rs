use crate::js::executable::{agent::AgentSignifier, environment::EnvironmentRecord};

pub struct Realm {
    pub agent_signifier: AgentSignifier,

    pub intrinsics: (),
    pub global_object: (),

    pub global_env: EnvironmentRecord,

    /// WARN: This likely will NOT be implemented!
    pub template_map: (),

    pub loaded_modules: Vec<()>,

    pub host_defined: (),
}
