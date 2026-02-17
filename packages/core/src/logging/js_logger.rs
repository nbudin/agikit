use std::cell::RefCell;

use log::Level;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
extern "C" {
    pub type LoggerTarget;

    #[wasm_bindgen(structural, method)]
    pub fn error(this: &LoggerTarget, message: &str);

    #[wasm_bindgen(structural, method)]
    pub fn warn(this: &LoggerTarget, message: &str);

    #[wasm_bindgen(structural, method)]
    pub fn log(this: &LoggerTarget, message: &str);
}

#[wasm_bindgen(typescript_custom_section)]
const LOGGER_TS_DEFINITION: &'static str = r#"
export interface Logger {
  error(message: string): void;
  warn(message: string): void;
  log(message: string): void;
}"#;

thread_local! {
  pub static JS_LOGGER_INSTANCE: RefCell<Option<LoggerTarget>> = RefCell::new(None);
}

pub struct JsLogger;
static JS_LOGGER_SINGLETON: JsLogger = JsLogger;

#[wasm_bindgen(js_name = "setLogger")]
pub fn set_logger(logger: LoggerTarget) -> Result<(), String> {
    JS_LOGGER_INSTANCE.set(Some(logger));
    log::set_logger(&JS_LOGGER_SINGLETON).map_err(|err| format!("{}", err))
}

impl log::Log for JsLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &log::Record) {
        JS_LOGGER_INSTANCE.with_borrow(|logger| match logger {
            Some(logger) => {
                let message = std::fmt::format(record.args().clone());
                match record.level() {
                    Level::Error => logger.error(&message),
                    Level::Warn => logger.warn(&message),
                    Level::Info => logger.log(&message),
                    Level::Debug | Level::Trace => {}
                }
            }
            None => {}
        });
    }

    fn flush(&self) {}
}
