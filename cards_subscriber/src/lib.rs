use tracing::{field::Visit, Id, Level};

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;

use std::fmt::Write as FmtWrite;
use std::fs::File;
use std::io::Write;

use colorful::Colorful;

struct MessageVisitor {
    message: String,
    field_name: &'static str,
}

impl MessageVisitor {
    fn new(field_name: &'static str) -> Self {
        Self {
            message: String::new(),
            field_name,
        }
    }
}

impl Visit for MessageVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == self.field_name {
            self.message = format!("{:?}", value);
        }
    }
}

#[derive(Debug)]
struct Attributes {
    name: String,
    target: String,
    level: Level,
    values: String,
}

impl From<&tracing::span::Attributes<'_>> for Attributes {
    fn from(attr: &tracing::span::Attributes<'_>) -> Self {
        let name = attr.metadata().name().to_string();
        let target = attr.metadata().target().to_string();
        let level = attr.metadata().level().clone();
        let mut values = attr
            .values()
            .to_string()
            .trim_matches(|c| c == '{' || c == '}')
            .to_string();
        if !values.is_empty() {
            values = format!("({})", values)
        }
        Self {
            name,
            target,
            level,
            values,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ApplyTo {
    Stdout,
    MainLog,
    SpecificLog,
    All,
}

// #[derive(Debug, Clone, Eq, PartialEq)]
pub enum TargetKind {
    Target(&'static str),
    Targets(Vec<&'static str>),
    Span(&'static str),
    Spans(Vec<&'static str>),
}

pub struct Filter {
    level: Option<Level>,
    target: Option<TargetKind>,
    apply_to: ApplyTo,
}

impl Filter {
    /// If the level is none, no messages will be shown for that target, for all messages set the level to Some(Level::TRACE)
    /// If the target is none, all aply
    pub fn new<>(level: Option<Level>, target: Option<TargetKind>, apply_to: ApplyTo) -> Self {
        Self {
            level,
            target,
            apply_to,
        }
    }

    fn should_show(&self, metadata: &tracing::Metadata<'_>, spans: &[&str]) -> Option<bool> {
        let should_show_level = match &self.level {
            None => false,
            Some(l) => metadata.level() <= l,
        };
        let should_show_target = match &self.target {
            None => true,
            Some(t) => match t {
                TargetKind::Target(t) => t == &metadata.target(),
                TargetKind::Targets(t) => t.contains(&metadata.target()),
                TargetKind::Span(s) => spans.contains(s),
                TargetKind::Spans(s) => s.iter().fold(false, |s, x| s || spans.contains(x))
            }
        };
        println!("spans: {:?}", spans);
    
        if should_show_target {
            Some(should_show_level)
        }else{
            None
        }
    }
}

pub struct Subscriber {
    id: Arc<RwLock<u64>>,
    stack: Arc<RwLock<Vec<Id>>>,
    spans: Arc<RwLock<HashMap<Id, Attributes>>>,
    show_args: bool,
    main_log: Arc<RwLock<File>>,
    target_logs: Arc<RwLock<HashMap<String, File>>>,
    filters: Arc<RwLock<Vec<Filter>>>,
}

impl Subscriber {
    pub fn new<P: AsRef<std::path::Path>>(
        logs_folder: P,
        custom_target_logs: &[&str],
        show_args: bool,
    ) -> Self {
        std::fs::create_dir_all(&logs_folder).unwrap();
        Self {
            id: Arc::new(RwLock::new(0)),
            stack: Default::default(),
            spans: Default::default(),
            filters: Default::default(),
            show_args,
            main_log: Arc::new(RwLock::new(
                File::create(logs_folder.as_ref().join("main.log")).unwrap(),
            )),
            target_logs: Arc::new(RwLock::new(
                custom_target_logs
                    .iter()
                    .map(|x| {
                        (
                            x.to_string(),
                            File::create(
                                logs_folder
                                    .as_ref()
                                    .join(format!("{}.log", x.replace("::", "."))),
                            )
                            .unwrap(),
                        )
                    })
                    .collect(),
            )),
        }
    }

    pub fn filter(self, filter: Filter) -> Self {
        self.filters.write().unwrap().push(filter);
        self
    }

    pub fn filter_mut(&self, filter: Filter) {
        self.filters.write().unwrap().push(filter);
    }

    fn show(&self, metadata: &tracing::Metadata<'_>, apply_to: ApplyTo) -> bool {
        let spans = self.spans.read().unwrap();
        let spans: Vec<&str> = self.stack.read().unwrap().iter().map(|x| spans[x].name.as_str()).collect();
        let mut res = None;
        let filters = self.filters.read().unwrap();
        let mut iter = filters.iter().filter(|x| x.apply_to == apply_to || x.apply_to == ApplyTo::All);
        while let (None, Some(v)) = (res, iter.next()) {
            res = v.should_show(metadata, &spans)
        }
        res.unwrap_or(true)
    }
}

impl tracing::Subscriber for Subscriber {
    fn enabled(&self, metadata: &tracing::Metadata<'_>) -> bool {
        
        self.show(metadata, ApplyTo::All)
    }

    fn new_span(&self, span: &tracing::span::Attributes<'_>) -> Id {
        *self.id.write().unwrap() += 1;
        let id = Id::from_u64(*self.id.read().unwrap());
        self.spans.write().unwrap().insert(id.clone(), span.into());
        id
    }

    fn record(&self, span: &Id, values: &tracing::span::Record<'_>) {
        println!("Recording [{:?}]: {:?}", span, values)
    }

    fn record_follows_from(&self, span: &Id, follows: &Id) {
        println!("{:?} follows {:?}", span, follows)
    }

    fn event(&self, event: &tracing::Event<'_>) {
        let time = chrono::Local::now();
        let mut visitor = MessageVisitor::new("message");
        event.record(&mut visitor);
        let mut colored_ctx = String::new();
        let mut ctx = String::new();
        for id in self.stack.read().unwrap().iter() {
            let attr = &self.spans.read().unwrap()[id];
            write!(
                colored_ctx,
                " [{}{}]",
                attr.name.clone().green(),
                if self.show_args { &attr.values } else { "" }
            )
            .unwrap();
            write!(ctx, " [{}{}]", attr.name, attr.values).unwrap();
        }
        if self.show(event.metadata(), ApplyTo::Stdout) {
            println!(
                "{} [{}] [{}] {} {}",
                format_level_colored(event.metadata().level()),
                time.format("%Y-%m-%d %H:%M:%S").to_string().light_gray(),
                event.metadata().target().blue(),
                colored_ctx.trim(),
                visitor.message
            );
        }
        let file_m = format!(
            "{} [{}] {} {}",
            format_level(event.metadata().level()),
            time.format("%Y-%m-%d %H:%M:%S"),
            ctx.trim(),
            visitor.message
        );
        if self.show(event.metadata(), ApplyTo::MainLog) {
            writeln!(self.main_log.write().unwrap(), "{}", file_m).unwrap();
            self.main_log.write().unwrap().flush().unwrap();
        }
        if self
            .target_logs
            .read()
            .unwrap()
            .contains_key(&event.metadata().target().to_string()) && self.show(event.metadata(), ApplyTo::SpecificLog)
        {
            let mut lock = self.target_logs.write().unwrap();
            let file = lock
                .get_mut(&event.metadata().target().to_string())
                .unwrap();
            writeln!(file, "{}", file_m).unwrap();
            file.flush().unwrap();
        }
    }

    fn enter(&self, span: &Id) {
        self.stack.write().unwrap().push(span.clone())
    }

    fn exit(&self, span: &Id) {
        self.stack.write().unwrap().retain(|x| x != span)
    }
}

fn format_level_colored(level: &Level) -> String {
    match level {
        &Level::TRACE => "TRACE".white(),
        &Level::DEBUG => "DEBUG".light_green(),
        &Level::INFO => "INFO ".light_cyan(),
        &Level::WARN => "WARN ".yellow(),
        &Level::ERROR => "ERROR".red(),
    }
    .to_string()
}

fn format_level(level: &Level) -> &'static str {
    match level {
        &Level::TRACE => "TRACE",
        &Level::DEBUG => "DEBUG",
        &Level::INFO => "INFO ",
        &Level::WARN => "WARN ",
        &Level::ERROR => "ERROR",
    }
}
