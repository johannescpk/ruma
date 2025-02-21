#![allow(dead_code)] // silence never-used warning for from_vec in generated code

xflags::xflags! {
    src "./src/flags.rs"

    /// Run custom task.
    cmd xtask {
        default cmd help {
            /// Print help information.
            optional -h, --help
        }

        /// Create a new release of the given crate.
        cmd release
            /// The crate to release
            required name: String
        {}

        /// Run CI tests.
        cmd ci
            optional version: String
        {}
    }
}
// generated start
// The following code is generated by `xflags` macro.
// Run `env UPDATE_XFLAGS=1 cargo build` to regenerate.
#[derive(Debug)]
pub struct Xtask {
    pub subcommand: XtaskCmd,
}

#[derive(Debug)]
pub enum XtaskCmd {
    Help(Help),
    Release(Release),
    Ci(Ci),
}

#[derive(Debug)]
pub struct Help {
    pub help: bool,
}

#[derive(Debug)]
pub struct Release {
    pub name: String,
}

#[derive(Debug)]
pub struct Ci {
    pub version: Option<String>,
}

impl Xtask {
    pub const HELP: &'static str = Self::HELP_;

    pub fn from_env() -> xflags::Result<Self> {
        Self::from_env_()
    }

    pub fn from_vec(args: Vec<std::ffi::OsString>) -> xflags::Result<Self> {
        Self::from_vec_(args)
    }
}
// generated end
