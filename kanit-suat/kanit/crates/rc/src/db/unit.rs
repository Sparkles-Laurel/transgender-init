use std::rc::Rc;

use async_trait::async_trait;
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
#[cfg(feature = "rkyv")]
use rkyv::Archive;

use kanit_common::error::{Context, ErrorKind, Result};
use kanit_supervisor::{RestartPolicy, Supervisor};
use kanit_unit::supervisor::SupervisorBuilder;
use kanit_unit::{Dependencies, Unit, UnitInfo, UnitName};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
#[cfg_attr(feature = "rkyv", derive(Archive, rkyv::Serialize, rkyv::Deserialize))]
#[cfg_attr(feature = "rkyv", archive(check_bytes))]
pub enum UnitKind {
    Oneshot,
    Daemon,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "rkyv", derive(Archive, rkyv::Serialize, rkyv::Deserialize))]
#[cfg_attr(feature = "rkyv", archive(check_bytes))]
pub struct DbUnit {
    pub name: UnitName,
    pub kind: UnitKind,
    pub description: Option<UnitName>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub before: Vec<UnitName>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub after: Vec<UnitName>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub needs: Vec<UnitName>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub uses: Vec<UnitName>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub wants: Vec<UnitName>,
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub supervisor_opts: Supervisor,
    #[cfg_attr(feature = "serde", serde(skip))]
    pid: u32,
}

impl DbUnit {
    pub fn get_unit_info(&self) -> UnitInfo {
        UnitInfo {
            name: self.name.clone(),
            dependencies: Rc::new(self.dependencies()),
        }
    }
}

#[async_trait]
impl Unit for DbUnit {
    fn name(&self) -> UnitName {
        self.name.clone()
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    fn dependencies(&self) -> Dependencies {
        let mut deps = Dependencies::new();

        self.before.iter().for_each(|b| {
            deps.before(b.clone());
        });
        self.after.iter().for_each(|b| {
            deps.after(b.clone());
        });
        self.needs.iter().for_each(|b| {
            deps.need(b.clone());
        });
        self.uses.iter().for_each(|b| {
            deps.uses(b.clone());
        });
        self.wants.iter().for_each(|b| {
            deps.want(b.clone());
        });

        deps
    }

    async fn start(&mut self) -> Result<()> {
        if self.kind == UnitKind::Oneshot {
            self.supervisor_opts.restart_policy = Some(RestartPolicy::OnFailure);
        }

        let child = SupervisorBuilder::from_supervisor(self.supervisor_opts.clone()).spawn()?;

        self.pid = child.id();

        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        kill(Pid::from_raw(self.pid as i32), Signal::SIGKILL)
            .context_kind("failed to stop supervisor", ErrorKind::Recoverable)
    }
}
