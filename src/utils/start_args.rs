#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StartArgs {
    pub(crate) target: String,
    pub(crate) args: Vec<String>,
    pub(crate) start_location: Option<String>,
}

impl StartArgs {
    pub fn args_to_string(args: &[String]) -> String {
        args.iter()
            .enumerate()
            .fold(String::new(), |mut s, (i, arg)| {
                if i > 0 {
                    s.push(' ');
                }
                s.push_str(arg);
                s
            })
    }

    pub fn build_from_args(mut dest_path: std::path::PathBuf, args: &crate::Opts) -> Self {
        let mut ret = Self {
            target: format!("{}", dest_path.display()),
            ..Default::default()
        };
        if dest_path.pop() {
            ret.start_location = dest_path.to_str().map(Into::into);
        }

        if let Some(timer_value) = args.timer {
            ret.args.push(format!("--timer {}", timer_value));
        }

        if args.clean_standby_list {
            ret.args.push("--islc".to_string());
            if args.clean_standby_list_poll_freq != 10 {
                ret.args.push(format!(
                    "--islc-timer {}",
                    args.clean_standby_list_poll_freq
                ));
            }
            if args.clear_standby_cached_mem != 1024 {
                ret.args
                    .push(format!("--cscm {}", args.clear_standby_cached_mem));
            }
            if args.clear_standby_free_mem != 1024 {
                ret.args
                    .push(format!("--csfm {}", args.clear_standby_free_mem));
            }
        }

        ret
    }
}

impl std::fmt::Display for StartArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\"{}\" {}",
            self.target,
            Self::args_to_string(&self.args)
        )
    }
}
