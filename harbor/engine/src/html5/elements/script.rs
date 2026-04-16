#![allow(dead_code)]

use std::{cell::RefCell, rc::Rc};

use crate::{
    html5::{
        cors::CORSSettingsAttr,
        dom::{Document, Element},
        environments::EnvironmentSettings,
        mime_types::IS_JS_MIME,
    },
    http::url::URL,
    js::{
        script::{ScriptRecord, parse_script, script_evaluation},
        values::ReferenceOrValue,
    },
};

#[derive(Debug, Clone)]
pub(crate) enum ScriptType {
    Null,
    Classic,
    Module,
    ImportMap,
    SpeculationRules,
}

pub(crate) type ScriptId = usize;

pub(crate) enum ScriptResult {
    Uninitialized,
    Error,
    Script(Script),
    ImportMap,
    SpeculationRules,
}

pub(crate) struct ScriptInternalState {
    parser_document: Option<Rc<RefCell<Document>>>,

    force_async: bool,
    from_external_file: bool,

    ready_to_be_executed: bool,
    already_started: bool,
    delay_load_event: bool,

    type_: ScriptType,
    result: ScriptResult,

    steps_to_run_when_ready: Vec<Box<dyn FnOnce()>>,
}

pub struct Script {
    pub settings: EnvironmentSettings,
    pub record: Option<ScriptRecord>,
    pub base_url: Option<URL>,
}

impl Script {
    pub fn classic(source: String, base_url: URL, settings: EnvironmentSettings) -> Self {
        let realm = settings.realm.clone().unwrap();
        let host_defined = realm.borrow().host_defined.clone();

        let script = Self {
            settings,
            record: None,
            base_url: Some(base_url),
        };

        let record = parse_script(&source, realm, &host_defined);

        Script {
            record: Some(record),
            ..script
        }
    }

    pub fn run(&mut self) -> ReferenceOrValue {
        let settings = &mut self.settings;
        settings.prepare_to_run();

        let eval_status = script_evaluation(Rc::new(self.record.clone().unwrap()));
        settings.cleanup_after_running();

        println!("Script evaluation completed with status: {:?}", eval_status);

        eval_status.value
    }
}

pub struct ScriptElement {
    pub raw: Rc<RefCell<Element>>,

    pub type_: Option<String>,
    pub src: Option<String>,

    pub async_: bool,
    pub defer: bool,

    pub blocking: Vec<String>,

    pub cross_origin: Option<String>,
    pub referrer_policy: Option<String>,

    pub integrity: Option<String>,
    pub fetch_priority: Option<String>,

    pub text: String,

    pub(crate) internal_state: ScriptInternalState,
}

impl ScriptElement {
    pub fn new(raw: Rc<RefCell<Element>>, parser: Option<Rc<RefCell<Document>>>) -> Self {
        let (
            type_,
            src,
            async_,
            defer,
            blocking,
            cross_origin,
            referrer_policy,
            integrity,
            fetch_priority,
            text,
        ) = {
            let raw_borrow = raw.borrow();

            let type_ = raw_borrow
                .get_attribute("type")
                .map(|t| t.to_lowercase().to_string());

            let src = raw_borrow.get_attribute("src").map(|s| s.to_string());

            let async_ = raw_borrow.has_attribute("async");
            let defer = raw_borrow.has_attribute("defer");

            let blocking = Vec::<String>::new();

            let cross_origin = raw_borrow
                .get_attribute("crossorigin")
                .map(|c| c.to_string());
            let referrer_policy = raw_borrow
                .get_attribute("referrerpolicy")
                .map(|r| r.to_string());

            let integrity = raw_borrow.get_attribute("integrity").map(|i| i.to_string());
            let fetch_priority = raw_borrow
                .get_attribute("fetchpriority")
                .map(|f| f.to_string());

            let text = raw_borrow.text_content().unwrap_or_default();

            (
                type_,
                src,
                async_,
                defer,
                blocking,
                cross_origin,
                referrer_policy,
                integrity,
                fetch_priority,
                text,
            )
        };

        Self {
            raw,

            type_,
            src,

            async_,
            defer,

            blocking,

            cross_origin,
            referrer_policy,

            integrity,
            fetch_priority,

            text,

            internal_state: ScriptInternalState {
                parser_document: parser,

                force_async: true,
                from_external_file: false,

                ready_to_be_executed: false,
                already_started: false,
                delay_load_event: false,

                type_: ScriptType::Null,
                result: ScriptResult::Uninitialized,

                steps_to_run_when_ready: Vec::new(),
            },
        }
    }

    pub fn prepare(&mut self) {
        if self.internal_state.already_started {
            return;
        }

        let parser_doc = self.internal_state.parser_document.clone();
        self.internal_state.parser_document = None;

        if parser_doc.is_some() && self.async_ {
            self.internal_state.force_async = true;
        }

        let source_text = &self.text;

        if self.src.is_none() && source_text.is_empty() {
            return;
        }

        let script_block_type = if self.type_.as_ref().is_some_and(|s| s.is_empty())
            || (self.type_.is_none()
                && self
                    .raw
                    .borrow()
                    .get_attribute("language")
                    .is_some_and(|s| s.is_empty()))
            || (self.type_.is_none() && self.raw.borrow().get_attribute("language").is_none())
        {
            "text/javascript".to_string()
        } else if self.type_.is_some() {
            self.type_.clone().unwrap().trim().to_string()
        } else {
            format!(
                "text/{}",
                self.raw.borrow().get_attribute("language").unwrap()
            )
        };

        self.internal_state.type_ = match script_block_type.to_lowercase().as_str() {
            "module" => ScriptType::Module,
            "importmap" => ScriptType::ImportMap,
            "speculationrules" => ScriptType::SpeculationRules,
            _ if IS_JS_MIME(&script_block_type) => ScriptType::Classic,
            _ => return,
        };

        if parser_doc.is_some() {
            self.internal_state.parser_document = parser_doc;
            self.internal_state.force_async = false;
        }

        self.internal_state.already_started = true;

        let classic_script_cors = self.cross_origin.as_ref().map(|c| c.to_lowercase());
        let cors_settings_attr = CORSSettingsAttr::from(classic_script_cors);
        // let credentials_mode = cors_settings_attr.credentials_mode();

        let node_document = self.raw.borrow().node_document().unwrap();
        let settings_obj = &node_document.borrow().settings;

        if self.src.is_none() {
            let base_url = self.raw.borrow()._node.borrow().base_uri();

            match self.internal_state.type_ {
                ScriptType::Classic => {
                    let script = Script::classic(
                        source_text.clone(),
                        URL::pure_parse(base_url).unwrap(),
                        settings_obj.clone(),
                    );

                    self.mark_as_ready(script);
                }
                _ => unimplemented!(
                    "Script type {:?} is not implemented yet",
                    self.internal_state.type_
                ),
            }
        }

        self.internal_state.ready_to_be_executed = true;
        self.execute();
    }

    pub fn execute(&mut self) {
        match &self.internal_state.type_ {
            ScriptType::Classic => {
                if let ScriptResult::Script(script) = &mut self.internal_state.result {
                    script.run();
                }
            }
            _ => unimplemented!(
                "Script type {:?} is not implemented yet",
                self.internal_state.type_
            ),
        }
    }

    pub fn mark_as_ready(&mut self, result: Script) {
        self.internal_state.result = ScriptResult::Script(result);
        if !self.internal_state.steps_to_run_when_ready.is_empty() {
            for step in self.internal_state.steps_to_run_when_ready.drain(..) {
                step();
            }
        }

        self.internal_state.delay_load_event = false;
    }

    pub fn supports(&self, type_: String) -> bool {
        if type_ == "classic" {
            return true;
        }

        // if type_ == "module" {
        //     return true;
        // }

        // if type_ == "importmap" {
        //     return true;
        // }

        // if type_ == "speculationrules" {
        //     return true;
        // }

        false
    }
}
