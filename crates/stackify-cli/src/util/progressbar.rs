use std::{borrow::Cow, fmt::Write, sync::Arc, time::Duration};

use color_eyre::Result;
use console::style;
use futures_util::Future;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};

pub fn new_progressbar(template: &str, message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(Duration::from_millis(200));
    pb.set_style(
        ProgressStyle::with_template(template)
            .unwrap()
            .tick_chars("/|\\- "),
    );
    pb.set_message(message.to_string());
    pb
}

pub enum PbType {
    Spinner,
    DownloadBar,
    ProgressBar
}

pub struct PbWrapper {
    pb: ProgressBar,
    title: Cow<'static, str>,
    pb_type: PbType
}

impl PbWrapper {
    pub fn new_download_bar(size: u64, title: impl Into<Cow<'static, str>>) -> Self {
        let title: Cow<'static, str> = title.into();
        let pb = ProgressBar::new(size);
        pb.enable_steady_tick(Duration::from_millis(100));
        pb.set_style(
            ProgressStyle::with_template(
                &format!(
                    "{{spinner:.cyan}} {} [{{elapsed_precise}}] [{{bar:40.cyan/blue}}] {{bytes}}/{{total_bytes}} ({{eta}})",
                    &title
                )
            )
            .unwrap()
            .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
            .progress_chars("#>-"));

        Self {
            pb,
            title,
            pb_type: PbType::DownloadBar
        }
    }

    pub fn new_progress_bar(size: u64, title: impl Into<Cow<'static, str>>) -> Self {
        let title: Cow<'static, str> = title.into();
        let pb = ProgressBar::new(size);
        pb.enable_steady_tick(Duration::from_millis(100));
        pb.set_style(
            ProgressStyle::with_template(
                &format!(
                    "{{spinner:.cyan}} {} [{{elapsed_precise}}] {{bar:40.cyan/blue}} ({{pos}}/{{len}})\n  {}{{msg}}", 
                    &title,
                    "🏁"
                )
            )
            .unwrap()
            .progress_chars("#>-"));

        Self {
            pb,
            title,
            pb_type: PbType::ProgressBar
        }
    }

    pub fn new_spinner(title: impl Into<Cow<'static, str>>) -> Self {
        let title: Cow<'static, str> = title.into();

        let pb = ProgressBar::new_spinner();
        pb.enable_steady_tick(Duration::from_millis(200));
        pb.set_style(
            ProgressStyle::with_template(
                &format!("{{spinner:.cyan}} {}: {{wide_msg}}", &title))
                .unwrap()
                //.tick_chars("/|\\- "),
        );

        Self {
            pb,
            title,
            pb_type: PbType::Spinner
        }
    }
}

/// Replace_with* methods.
#[allow(dead_code)]
impl PbWrapper {
    pub fn replace_with_spinner(&mut self) {
        self.pb.finish_and_clear();
        
        let pbw = Self::new_spinner(self.title.clone());

        self.pb = pbw.pb;
        self.pb_type = PbType::Spinner;
    }

    pub fn replace_with_download_bar(&mut self, size: u64) {
        self.pb.finish_and_clear();
        
        let pbw = Self::new_download_bar(size, self.title.clone());

        self.pb = pbw.pb;
        self.pb_type = PbType::DownloadBar;
    }

    pub fn replace_with_progress_bar(&mut self, size: u64) {
        self.pb.finish_and_clear();
        
        let pbw = Self::new_progress_bar(size, self.title.clone());

        self.pb = pbw.pb;
        self.pb_type = PbType::ProgressBar;
    }
}

/// Progress methods.
impl PbWrapper {
    pub fn set_length(&self, size: u64) {
        self.pb.set_length(size);
    }

    // pub fn set_title(&mut self, title: impl Into<Cow<'static, str>>) {
    //     self.title = title.into();

    //     match self.pb_type {
    //         PbType::Spinner => {
    //             self.pb.set_style(
    //                 ProgressStyle::with_template(
    //                     &format!("{{spinner:.cyan}} {}: {{wide_msg}}", &self.title))
    //                     .unwrap()
    //                     //.tick_chars("/|\\- "),
    //             );
    //         },
    //         PbType::ProgressBar => {
    //             self.pb.set_style(
    //                 ProgressStyle::with_template(
    //                     &format!(
    //                         "{} {{spinner:.cyan}} [{{elapsed_precise}}] [{{bar:40.cyan/blue}}] {{bytes}}/{{total_bytes}} ({{eta}})",
    //                         &self.title
    //                     )
    //                 )
    //                 .unwrap()
    //                 .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
    //                 .progress_chars("#>-"));
    //         }
    //     }
    // }

    pub fn set_message(&self, msg: impl Into<Cow<'static, str>>) {
        self.pb.set_message(msg);
    }

    pub fn exec<T>(&mut self, f: impl FnOnce(&mut PbWrapper) -> Result<T>) -> Result<T> {
        let result = f(self);
        self.pb.finish_and_clear();
        match result {
            Ok(_) => println!("{} {}", style("✔️").green(), self.title),
            Err(_) => println!("{} {}", style("⨯").red(), self.title)
        };
    
        result
    }

    pub fn finish_success(&self) {
        self.pb.finish_and_clear();
        println!("{} {}", 
            style("✔️").green(), 
            self.title
        );
    }

    pub fn finish_fail(&self) {
        self.pb.finish_and_clear();
        println!("{} {}", 
            style("⨯").red(), 
            self.title
        );
    }

    pub async fn exec_async<T, Fut>(
        &mut self, 
        f: impl FnOnce(&mut PbWrapper) -> Fut
    ) -> Result<T> 
    where
        Fut: Future<Output = Result<T>>
    {
        let result = f(self).await;
        self.pb.finish_and_clear();
        match result {
            Ok(_) => println!("{} {}", style("✔️").green(), self.title),
            Err(_) => println!("{} {}", style("⨯").red(), self.title)
        };
    
        result
    }

    pub fn inc(&self, by: u64) {
        self.pb.inc(by)
    }

    pub fn inc_length(&self, by: u64) {
        self.pb.inc_length(by)
    }

    pub fn get_length(&self) -> Option<u64> {
        self.pb.length()
    }

    pub fn get_position(&self) -> u64 {
        self.pb.position()
    }
}